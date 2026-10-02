//! Explicit, resumable ingestion. Never uses the interactive search result cap.
use crate::{
    budget,
    progress::{IngestPhase, Progress, ProgressCallback},
};
use crate::{
    error,
    manifest::{atomic_json, resolve_file, DataFile, DatasetManifest, IngestScope, Snapshot},
};
use chrono::Utc;
use fs2::FileExt;
use reqwest::{Client, Method, Url};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use stac::{
    api::{Search, UrlBuilder},
    Item, ItemCollection, Link,
};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    time::Duration,
};
use superstac_core::{errors::SuperSTACError, models::catalog::Catalog};

#[derive(Clone)]
pub struct IngestOptions {
    pub output: PathBuf,
    pub scope: IngestScope,
    /// Requested server page size, not a total result limit.
    pub page_size: usize,
    /// Approximate target: flush at page boundaries and split by collection.
    pub items_per_file: usize,
    pub request_timeout: Duration,
    pub resume: bool,
    /// Named scope to refresh. Omit to derive a stable name from normalized coverage.
    pub name: Option<String>,
    /// Includes current, old, and incomplete generations under runs/. No eviction.
    pub max_dataset_bytes: Option<u64>,
    pub progress: Option<ProgressCallback>,
}

impl IngestOptions {
    pub fn new(output: impl Into<PathBuf>, scope: IngestScope) -> Self {
        Self {
            output: output.into(),
            scope,
            page_size: 500,
            items_per_file: 10_000,
            request_timeout: Duration::from_secs(60),
            resume: false,
            name: None,
            max_dataset_bytes: Some(1024 * 1024 * 1024),
            progress: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct PageRequest {
    url: String,
    method: String,
    body: Option<Map<String, Value>>,
    headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Checkpoint {
    version: u32,
    snapshot: Snapshot,
    initial: PageRequest,
    next: Option<PageRequest>,
    run: PathBuf,
    last_error: Option<String>,
    #[serde(default)]
    visited_requests: HashSet<String>,
}

/// Ingest or refresh one named scope, retaining other scopes for the catalog.
pub async fn ingest_catalog(
    catalog: &Catalog,
    options: IngestOptions,
) -> Result<Snapshot, SuperSTACError> {
    let mut progress = Progress::new(&catalog.id, options.progress.clone());
    progress.emit(IngestPhase::Started, None);
    let result = ingest_inner(catalog, options, &mut progress).await;
    match &result {
        Ok(snapshot) => {
            progress.saved(snapshot);
            progress.emit(IngestPhase::Completed, None);
        }
        Err(e) => progress.emit(IngestPhase::Failed, Some(e.to_string())),
    }
    result
}

async fn ingest_inner(
    catalog: &Catalog,
    options: IngestOptions,
    progress: &mut Progress,
) -> Result<Snapshot, SuperSTACError> {
    if options
        .name
        .as_ref()
        .is_some_and(|name| name.trim().is_empty() || name.len() > 128)
    {
        return Err(error("scope name must be nonempty and at most 128 bytes"));
    }
    if options.page_size == 0
        || options.page_size > 10_000
        || options.items_per_file == 0
        || options.items_per_file > 1_000_000
        || options.request_timeout.is_zero()
    {
        return Err(error(
            "page_size must be 1..=10000, items_per_file 1..=1000000, and timeout positive",
        ));
    }
    if options.scope.bbox.is_some_and(|b| {
        !matches!(b, stac::Bbox::TwoDimensional(_))
            || !b.is_valid()
            || Vec::<f64>::from(b).iter().any(|v| !v.is_finite())
    }) {
        return Err(error(
            "ingestion requires a finite 2D bbox that does not cross the antimeridian",
        ));
    }
    let mut scope = options.scope.clone();
    scope.collections = scope
        .collections
        .iter()
        .map(|c| catalog.resolve_collection(c).to_owned())
        .collect();
    scope.collections.sort();
    scope.collections.dedup();
    let mut search = Search::new()
        .collections(scope.collections.clone())
        .limit(options.page_size as u64);
    search.items.bbox = scope.bbox;
    search.items.datetime = scope.datetime;
    let search = search
        .normalize_datetimes()
        .map_err(error)?
        .valid()
        .map_err(error)?;
    scope.datetime = search.items.datetime.clone();
    let initial = PageRequest {
        url: UrlBuilder::new(&catalog.url)
            .map_err(error)?
            .search()
            .to_string(),
        method: "POST".into(),
        body: Some(
            serde_json::to_value(search)
                .map_err(error)?
                .as_object()
                .ok_or_else(|| error("invalid search request"))?
                .clone(),
        ),
        headers: BTreeMap::new(),
    };
    fs::create_dir_all(&options.output).map_err(error)?;
    let root = options.output.canonicalize().map_err(error)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join(".writer.lock"))
        .map_err(error)?;
    lock.try_lock_exclusive()
        .map_err(|e| error(format!("dataset already being written or cannot lock: {e}")))?;
    fs::create_dir_all(root.join("checkpoints")).map_err(error)?;
    fs::create_dir_all(root.join("runs")).map_err(error)?;
    let manifest = if root.join("manifest.json").exists() {
        DatasetManifest::read(&root)?
    } else {
        DatasetManifest::default()
    };
    let existing = manifest.catalogs.get(&catalog.id);
    let default_name = format!(
        "scope-{:x}",
        Sha256::digest(serde_json::to_vec(&scope).map_err(error)?)
    );
    let name = options
        .name
        .clone()
        .or_else(|| {
            existing.and_then(|scopes| {
                scopes
                    .values()
                    .find(|snapshot| snapshot.source_url == catalog.url && snapshot.scope == scope)
                    .map(|snapshot| snapshot.name.clone())
            })
        })
        .unwrap_or(default_name);
    let key = format!(
        "{:x}",
        Sha256::digest(format!("{}\0{name}", catalog.id).as_bytes())
    );
    let mut checkpoint_path = root.join("checkpoints").join(format!("{key}.json"));
    // Keep interrupted v1 jobs resumable when the original scope was unnamed.
    let legacy_checkpoint = root
        .join("checkpoints")
        .join(format!("{}.json", encode(&catalog.id)));
    if options.resume
        && options.name.is_none()
        && !checkpoint_path.exists()
        && legacy_checkpoint.exists()
    {
        checkpoint_path = legacy_checkpoint;
    }
    let mut checkpoint = if options.resume && checkpoint_path.exists() {
        let mut saved: Checkpoint =
            serde_json::from_reader(File::open(&checkpoint_path).map_err(error)?).map_err(error)?;
        if saved.version != 1
            || saved.snapshot.catalog_id != catalog.id
            || saved.snapshot.source_url != catalog.url
            || saved.snapshot.scope != scope
            || saved.initial != initial
        {
            return Err(error("checkpoint differs from this request; resume with the original scope and page size"));
        }
        saved.snapshot.name = name.clone();
        for file in &saved.snapshot.files {
            resolve_file(&root, &file.path)?;
        }
        let run = root.join(&saved.run).canonicalize().map_err(error)?;
        if !run.starts_with(root.join("runs")) {
            return Err(error("invalid checkpoint run directory"));
        }
        saved
    } else {
        if options.resume && root.join("manifest.json").exists() {
            let manifest = DatasetManifest::read(&root)?;
            if let Some(snapshot) = manifest
                .catalogs
                .get(&catalog.id)
                .and_then(|scopes| scopes.get(&name))
            {
                if snapshot.source_url == catalog.url && snapshot.scope == scope {
                    for file in &snapshot.files {
                        resolve_file(&root, &file.path)?;
                    }
                    progress.resume(snapshot);
                    return Ok(snapshot.clone());
                }
            }
        }
        if options.resume {
            return Err(error(
                "no matching checkpoint or completed snapshot to resume",
            ));
        }
        let run = tempfile::Builder::new()
            .prefix("snapshot-")
            .tempdir_in(root.join("runs"))
            .map_err(error)?
            .keep();
        #[cfg(unix)]
        File::open(root.join("runs"))
            .and_then(|f| f.sync_all())
            .map_err(error)?;
        Checkpoint {
            version: 1,
            snapshot: Snapshot {
                name: name.clone(),
                catalog_id: catalog.id.clone(),
                source_url: catalog.url.clone(),
                scope,
                started_at: Utc::now(),
                completed_at: None,
                complete: false,
                pages: 0,
                items: 0,
                files: Vec::new(),
            },
            initial: initial.clone(),
            next: Some(initial),
            run: run.strip_prefix(&root).map_err(error)?.to_owned(),
            last_error: None,
            visited_requests: HashSet::new(),
        }
    };
    progress.resume(&checkpoint.snapshot);
    atomic_json(&checkpoint_path, &checkpoint)?;
    let client = Client::builder()
        .user_agent(concat!("superstac/", env!("CARGO_PKG_VERSION")))
        .timeout(options.request_timeout)
        .build()
        .map_err(error)?;
    let result = download(
        &client,
        &root,
        &checkpoint_path,
        &mut checkpoint,
        &options,
        progress,
    )
    .await;
    if let Err(e) = result {
        // Save diagnostics against the last durable checkpoint, not buffered pages.
        if let Ok(file) = File::open(&checkpoint_path) {
            if let Ok(mut durable) = serde_json::from_reader::<_, Checkpoint>(file) {
                durable.last_error = Some(e.to_string());
                let _ = atomic_json(&checkpoint_path, &durable);
            }
        }
        return Err(e);
    }
    checkpoint.snapshot.complete = true;
    checkpoint.snapshot.completed_at = Some(Utc::now());
    let mut manifest = if root.join("manifest.json").exists() {
        DatasetManifest::read(&root)?
    } else {
        DatasetManifest::default()
    };
    manifest
        .catalogs
        .entry(catalog.id.clone())
        .or_default()
        .insert(name, checkpoint.snapshot.clone());
    atomic_json(&root.join("manifest.json"), &manifest)?;
    fs::remove_file(checkpoint_path).map_err(error)?;
    // `lock` remains alive through publication and is released on drop/process exit.
    drop(lock);
    Ok(checkpoint.snapshot)
}

async fn download(
    client: &Client,
    root: &Path,
    checkpoint_path: &Path,
    state: &mut Checkpoint,
    options: &IngestOptions,
    progress: &mut Progress,
) -> Result<(), SuperSTACError> {
    let mut buffer = Vec::new();
    let mut buffered_bytes = 0usize;
    let mut remaining_bytes = match options.max_dataset_bytes {
        Some(max) => max
            .checked_sub(budget::used_bytes(&root.join("runs"))?)
            .ok_or_else(|| {
                error("existing data exceeds storage budget; raise --max-dataset-mib")
            })?,
        None => u64::MAX,
    };
    if remaining_bytes == 0 && state.next.is_some() {
        return Err(error(
            "dataset storage budget exhausted; raise --max-dataset-mib and resume",
        ));
    }
    while let Some(request) = state.next.clone() {
        if !state
            .visited_requests
            .insert(serde_json::to_string(&request).map_err(error)?)
        {
            return Err(error(
                "provider returned a pagination cycle; snapshot remains incomplete",
            ));
        }
        progress.emit(IngestPhase::Fetching, None);
        let (page, response_url, bytes) = fetch_page(client, &request, progress).await?;
        progress.event.items_received += page.items.len() as u64;
        progress.event.pages += 1;
        progress.event.total_items = page
            .additional_fields
            .get("numberMatched")
            .and_then(Value::as_u64)
            .or_else(|| {
                page.additional_fields
                    .get("context")
                    .and_then(|c| c.get("matched"))
                    .and_then(Value::as_u64)
            })
            .or(progress.event.total_items);
        progress.emit(IngestPhase::Fetching, None);
        let next_links: Vec<_> = page
            .links
            .iter()
            .filter(|link| link.rel == "next")
            .collect();
        if next_links.len() > 1 {
            return Err(error("provider returned multiple next links"));
        }
        state.next = next_links
            .first()
            .map(|link| next_request(link, &response_url, &state.initial))
            .transpose()?;
        state.snapshot.pages += 1;
        buffered_bytes += bytes;
        for mut item in page.items {
            let base = item
                .links
                .iter()
                .find(|link| link.rel == "self")
                .map(|link| response_url.join(&link.href))
                .transpose()
                .map_err(error)?
                .unwrap_or_else(|| response_url.clone());
            for asset in item.assets.values_mut() {
                asset.href = stac::href::make_absolute(&asset.href, base.as_str())
                    .map_err(error)?
                    .into_owned();
            }
            for link in &mut item.links {
                if link.rel == "self" {
                    link.href = base.to_string();
                } else {
                    link.make_absolute(base.as_str()).map_err(error)?;
                }
            }
            buffer.push(item);
        }
        // Empty pages can still carry continuation links and must not end ingestion.
        if buffer.len() >= options.items_per_file
            || buffered_bytes >= 64 * 1024 * 1024
            || state.next.is_none()
        {
            let rows = std::mem::take(&mut buffer);
            let run = root.join(&state.run);
            let dataset_root = root.to_owned();
            progress.emit(IngestPhase::Writing, None);
            let files = tokio::task::spawn_blocking(move || {
                write_files(&dataset_root, &run, rows, remaining_bytes)
            })
            .await
            .map_err(error)??;
            remaining_bytes =
                remaining_bytes.saturating_sub(files.iter().map(|f| f.bytes).sum::<u64>());
            state.snapshot.items += files.iter().map(|f| f.items).sum::<u64>();
            state.snapshot.files.extend(files);
            state.last_error = None;
            atomic_json(checkpoint_path, state)?;
            progress.saved(&state.snapshot);
            progress.emit(IngestPhase::Checkpointed, None);
            buffered_bytes = 0;
        }
    }
    Ok(())
}

async fn fetch_page(
    client: &Client,
    request: &PageRequest,
    progress: &mut Progress,
) -> Result<(ItemCollection, Url, usize), SuperSTACError> {
    let method: Method = request.method.parse().map_err(error)?;
    if method != Method::GET && method != Method::POST {
        return Err(error("pagination supports GET and POST only"));
    }
    let mut last_error = String::new();
    for attempt in 0..3 {
        let mut builder = client.request(method.clone(), &request.url);
        for (key, value) in &request.headers {
            builder = builder.header(key, value);
        }
        if let Some(body) = &request.body {
            builder = if method == Method::GET {
                let fields: Vec<_> = body
                    .iter()
                    .map(|(key, value)| {
                        let value = match value {
                            Value::String(s) => s.clone(),
                            Value::Array(values) => values
                                .iter()
                                .map(|v| {
                                    v.as_str()
                                        .map(str::to_owned)
                                        .unwrap_or_else(|| v.to_string())
                                })
                                .collect::<Vec<_>>()
                                .join(","),
                            value => value.to_string(),
                        };
                        (key, value)
                    })
                    .collect();
                builder.query(&fields)
            } else {
                builder.json(body)
            };
        }
        match builder.send().await {
            Ok(mut response) => {
                let status = response.status();
                if status.is_success() {
                    let url = response.url().clone();
                    let mut body = Vec::new();
                    while let Some(chunk) = response.chunk().await.map_err(error)? {
                        if body.len() + chunk.len() > 32 * 1024 * 1024 {
                            return Err(error("API page exceeds 32 MiB; reduce --page-size and restart without --resume"));
                        }
                        body.extend_from_slice(&chunk);
                    }
                    let page = serde_json::from_slice(&body).map_err(error)?;
                    return Ok((page, url, body.len()));
                }
                if status.as_u16() != 429 && !status.is_server_error() {
                    return Err(error(format!("API returned {status}; checkpoint retained")));
                }
                last_error = format!("API returned {status}");
                if attempt < 2 {
                    let delay = response
                        .headers()
                        .get("retry-after")
                        .and_then(|h| h.to_str().ok())
                        .and_then(|s| {
                            s.parse::<u64>().ok().or_else(|| {
                                chrono::DateTime::parse_from_rfc2822(s).ok().map(|date| {
                                    (date.with_timezone(&Utc) - Utc::now()).num_seconds().max(0)
                                        as u64
                                })
                            })
                        })
                        .unwrap_or(1 << attempt);
                    if delay > 60 {
                        return Err(error(format!(
                            "provider requested a {delay}s pause; resume ingestion later"
                        )));
                    }
                    progress.emit(
                        IngestPhase::Retrying,
                        Some(format!("{status}; retrying in {delay}s")),
                    );
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                }
            }
            Err(e) => {
                last_error = e.without_url().to_string();
                if attempt < 2 {
                    progress.emit(
                        IngestPhase::Retrying,
                        Some(format!("request failed; retrying in {}s", 1 << attempt)),
                    );
                    tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
                }
            }
        }
    }
    Err(error(format!(
        "ingestion request failed after 3 attempts: {last_error}"
    )))
}

fn next_request(
    link: &Link,
    response_url: &Url,
    initial: &PageRequest,
) -> Result<PageRequest, SuperSTACError> {
    let url = response_url.join(&link.href).map_err(error)?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(error("pagination link must use HTTP(S)"));
    }
    let mut body = if link.merge.unwrap_or(false) {
        initial.body.clone().unwrap_or_default()
    } else {
        Map::new()
    };
    if let Some(extra) = &link.body {
        body.extend(extra.clone());
    }
    let mut headers = if link.merge.unwrap_or(false) {
        initial.headers.clone()
    } else {
        BTreeMap::new()
    };
    if let Some(extra) = &link.headers {
        for (key, value) in extra {
            headers.insert(
                key.clone(),
                value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| value.to_string()),
            );
        }
    }
    Ok(PageRequest {
        url: url.to_string(),
        method: link.method.clone().unwrap_or_else(|| "GET".into()),
        body: (!body.is_empty()).then_some(body),
        headers,
    })
}

fn write_files(
    root: &Path,
    run: &Path,
    rows: Vec<Item>,
    mut remaining: u64,
) -> Result<Vec<DataFile>, SuperSTACError> {
    let mut groups: BTreeMap<Option<String>, Vec<Item>> = BTreeMap::new();
    for item in rows {
        groups
            .entry(item.collection.clone())
            .or_default()
            .push(item);
    }
    let mut files = Vec::new();
    for (collection, items) in groups {
        let directory = run.join(format!(
            "collection={}",
            collection
                .as_deref()
                .map(encode)
                .unwrap_or_else(|| "none".into())
        ));
        fs::create_dir_all(&directory).map_err(error)?;
        #[cfg(unix)]
        File::open(run).and_then(|f| f.sync_all()).map_err(error)?;
        let file = tempfile::Builder::new()
            .prefix("part-")
            .suffix(".parquet")
            .tempfile_in(&directory)
            .map_err(error)?;
        let count = items.len() as u64;
        let mut bbox = None;
        let mut all_bbox = true;
        let mut datetime_min = None;
        let mut datetime_max = None;
        let mut all_time = true;
        for item in &items {
            match item.bbox {
                Some(b) if matches!(b, stac::Bbox::TwoDimensional(_)) && b.is_valid() => {
                    if let Some(bounds) = &mut bbox {
                        stac::Bbox::update(bounds, b);
                    } else {
                        bbox = Some(b);
                    }
                }
                _ => all_bbox = false,
            }
            let start = item.properties.start_datetime.or(item.properties.datetime);
            let end = item.properties.end_datetime.or(item.properties.datetime);
            match (start, end) {
                (Some(start), Some(end)) => {
                    datetime_min =
                        Some(datetime_min.map_or(start, |v: chrono::DateTime<Utc>| v.min(start)));
                    datetime_max =
                        Some(datetime_max.map_or(end, |v: chrono::DateTime<Utc>| v.max(end)));
                }
                _ => all_time = false,
            }
        }
        // Infer across the whole bounded file, preserving source-local properties.
        stac::geoparquet::WriterBuilder::new(budget::BudgetWriter::new(
            file.as_file(),
            &mut remaining,
        ))
        .options(stac::geoarrow::Options {
            drop_invalid_attributes: false,
        })
        .build(items)
        .map_err(error)?
        .finish()
        .map_err(error)?;
        file.as_file().sync_all().map_err(error)?;
        let bytes = file.as_file().metadata().map_err(error)?.len();
        let (_, path) = file.keep().map_err(error)?;
        #[cfg(unix)]
        File::open(&directory)
            .and_then(|f| f.sync_all())
            .map_err(error)?;
        files.push(DataFile {
            path: path.strip_prefix(root).map_err(error)?.to_owned(),
            collection,
            items: count,
            bytes,
            bbox: all_bbox.then_some(bbox).flatten(),
            datetime_min: all_time.then_some(datetime_min).flatten(),
            datetime_max: all_time.then_some(datetime_max).flatten(),
        });
    }
    Ok(files)
}

// IDs are encoded, never interpreted as filesystem paths.
fn encode(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
