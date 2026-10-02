//! Searches local STAC GeoParquet files or managed datasets with coverage manifests.
//!
//! Files retain source-local collection and asset names. Searches scan bounded
//! batches on Tokio's blocking pool with conservative row-group pruning. Managed
//! snapshots support coverage unions, ordered overlays, and explicit maintenance.
pub mod auto;
mod budget;
mod coverage;
pub mod ingest;
pub mod maintenance;
pub mod manifest;
pub mod progress;
mod pruning;
mod sorting;

use std::{
    collections::HashMap,
    fs::File,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use arrow_array::RecordBatchIterator;
use futures::future::BoxFuture;
use geoparquet::reader::{GeoParquetReaderBuilder, GeoParquetRecordBatchReader};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use superstac_core::{errors::SuperSTACError, models::catalog::Catalog};
use superstac_search::{
    backend::{BackendSearchOptions, SearchBackend},
    query::SearchQuery,
    response::SearchItem,
    translator::to_stac_search,
    unifier,
};

/// Maps configured catalog IDs to local STAC GeoParquet files.
/// No network requests or provider health checks are made by this backend.
pub struct GeoParquetBackend {
    paths: HashMap<String, Vec<PathBuf>>,
    snapshots:
        std::collections::BTreeMap<String, std::collections::BTreeMap<String, manifest::Snapshot>>,
    root: Option<PathBuf>,
    _reader_lock: Option<Arc<File>>,
}

impl GeoParquetBackend {
    pub fn new(paths: HashMap<String, PathBuf>) -> Result<Self, SuperSTACError> {
        if paths.is_empty() {
            return Err(error("at least one catalog snapshot is required"));
        }
        for (id, path) in &paths {
            if !path.is_file() {
                return Err(error(format!(
                    "catalog '{id}': not a local file: {}",
                    path.display()
                )));
            }
        }
        Ok(Self {
            paths: paths
                .into_iter()
                .map(|(id, path)| (id, vec![path]))
                .collect(),
            snapshots: Default::default(),
            root: None,
            _reader_lock: None,
        })
    }
    /// Opens one committed dataset generation; readers retain it during refreshes.
    pub fn from_dataset(root: impl AsRef<std::path::Path>) -> Result<Self, SuperSTACError> {
        let root = root.as_ref();
        let reader_lock = maintenance::lock(root, ".readers.lock", false)?;
        let manifest = manifest::DatasetManifest::read(root)?;
        if manifest.catalogs.is_empty() {
            return Err(error("dataset has no completed snapshots"));
        }
        Ok(Self {
            paths: HashMap::new(),
            snapshots: manifest.catalogs,
            root: Some(root.canonicalize().map_err(error)?),
            _reader_lock: Some(Arc::new(reader_lock)),
        })
    }

    pub fn catalog_ids(&self) -> impl Iterator<Item = &String> {
        self.paths.keys().chain(self.snapshots.keys())
    }

    /// Collection summaries describe the saved inventory, not the provider's full archive.
    pub fn collections(
        &self,
        catalog: &Catalog,
    ) -> Result<std::collections::BTreeMap<String, stac::Collection>, SuperSTACError> {
        let mut collections = std::collections::BTreeMap::new();
        let mut paths = Vec::new();
        if let Some(root) = &self.root {
            for snapshot in self
                .snapshots
                .get(&catalog.id)
                .into_iter()
                .flat_map(|s| s.values())
                .filter(|s| s.source_url == catalog.url)
            {
                for id in &snapshot.scope.collections {
                    collections.entry(catalog.canonical_collection(id).to_owned()).or_insert_with(|| stac::Collection::new(catalog.canonical_collection(id), "Saved SuperSTAC inventory; extent describes locally stored metadata only"));
                }
                for file in &snapshot.files {
                    paths.push(manifest::resolve_file(root, &file.path)?);
                }
            }
        } else {
            paths.extend(self.paths.get(&catalog.id).into_iter().flatten().cloned());
        }
        paths.sort();
        paths.dedup();
        let mut initialized = std::collections::HashSet::new();
        for path in paths {
            visit_items(&path, None, &AtomicBool::new(false), |item| {
                if let Some(id) = &item.collection {
                    let id = catalog.canonical_collection(id).to_owned();
                    if initialized.insert(id.clone()) {
                        collections.insert(id.clone(), stac::Collection::new_from_item(&id, "Saved SuperSTAC inventory; extent describes locally stored metadata only", &item));
                    } else if let Some(collection) = collections.get_mut(&id) {
                        collection.add_item(&item);
                        collection.links.clear();
                    }
                }
                Ok(true)
            })?;
        }
        Ok(collections)
    }

    fn select_snapshots<'a>(
        &'a self,
        catalog: &Catalog,
        search: &stac::api::Search,
        max_age: Option<std::time::Duration>,
    ) -> Result<Vec<&'a manifest::Snapshot>, SuperSTACError> {
        let mut scopes: Vec<_> = self
            .snapshots
            .get(&catalog.id)
            .into_iter()
            .flat_map(|s| s.values())
            .filter(|s| {
                s.source_url == catalog.url
                    && max_age.is_none_or(|max| {
                        chrono::Utc::now()
                            .signed_duration_since(s.started_at)
                            .to_std()
                            .ok()
                            .is_some_and(|age| age <= max)
                    })
            })
            .collect();
        scopes.sort_by(|a, b| (b.started_at, &b.name).cmp(&(a.started_at, &a.name)));
        if let Some(scope) = scopes.iter().find(|s| s.ensure_coverage(search).is_ok()) {
            return Ok(vec![*scope]);
        }
        if coverage::covers(&scopes, search)? {
            Ok(scopes)
        } else {
            Ok(vec![])
        }
    }

    pub fn can_answer(
        &self,
        catalog: &Catalog,
        query: SearchQuery,
        max_age: std::time::Duration,
    ) -> Result<bool, SuperSTACError> {
        Ok(!self
            .select_snapshots(catalog, &prepare_query(catalog, query)?, Some(max_age))?
            .is_empty())
    }

    pub(crate) async fn search_with_age(
        &self,
        catalog: &Catalog,
        query: SearchQuery,
        options: BackendSearchOptions,
        max_age: Option<std::time::Duration>,
    ) -> Result<Vec<SearchItem>, SuperSTACError> {
        let cap = query.limit.unwrap_or(10).min(options.max_items_per_catalog);
        let search = prepare_query(catalog, query)?;
        let mut overlays = false;
        let paths = if let Some(root) = &self.root {
            let snapshots = self.select_snapshots(catalog, &search, max_age)?;
            if snapshots.is_empty() {
                return Err(error(format!("query exceeds recorded coverage, source differs, or snapshot is stale for catalog '{}'", catalog.id)));
            }
            overlays = snapshots.len() > 1 || snapshots.iter().any(|s| s.overlays);
            let mut paths = Vec::new();
            for snapshot in snapshots {
                for file in &snapshot.files {
                    if (overlays || file.may_match(&search)?)
                        && !paths.contains(&root.join(&file.path))
                    {
                        paths.push(manifest::resolve_file(root, &file.path)?);
                    }
                }
            }
            paths
        } else {
            self.paths
                .get(&catalog.id)
                .ok_or_else(|| error(format!("no snapshot for catalog '{}'", catalog.id)))?
                .clone()
        };
        if cap == 0 {
            return Ok(Vec::new());
        }
        let catalog = catalog.clone();
        let reader_lease = self._reader_lock.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let _cancel_on_drop = CancelOnDrop(cancelled.clone());
        tokio::task::spawn_blocking(move || {
            let _reader_lease = reader_lease;
            let mut results = Vec::new();
            let mut seen = std::collections::HashSet::new();
            let sorted = !search.items.sortby.is_empty();
            for path in paths {
                visit_items(
                    &path,
                    if overlays { None } else { Some(&search) },
                    &cancelled,
                    |mut item| {
                        if !seen.insert((item.collection.clone(), item.id.clone())) {
                            return Ok(true);
                        }
                        if search.matches(&item).map_err(error)? {
                            if options.unify_response {
                                unifier::unify_item(&mut item, &catalog);
                            }
                            let result = SearchItem {
                                catalog_id: catalog.id.clone(),
                                seen_in: vec![catalog.id.clone()],
                                item,
                            };
                            if sorted {
                                sorting::insert(&mut results, result, &search.items.sortby, cap)?;
                            } else {
                                results.push(result);
                            }
                        }
                        Ok(sorted || results.len() < cap)
                    },
                )?;
                if !sorted && results.len() >= cap {
                    break;
                }
            }
            Ok::<_, SuperSTACError>(results)
        })
        .await
        .map_err(error)?
    }
}

impl SearchBackend for GeoParquetBackend {
    fn search<'a>(
        &'a self,
        catalog: &'a Catalog,
        query: SearchQuery,
        options: BackendSearchOptions,
    ) -> BoxFuture<'a, Result<Vec<SearchItem>, SuperSTACError>> {
        Box::pin(self.search_with_age(catalog, query, options, None))
    }
}

fn prepare_query(
    catalog: &Catalog,
    mut query: SearchQuery,
) -> Result<stac::api::Search, SuperSTACError> {
    if query.sortby.as_ref().is_some_and(|fields| {
        fields
            .iter()
            .any(|f| f.trim_start_matches(['+', '-']).is_empty())
    }) {
        return Err(error("sortby fields must be nonempty"));
    }
    query.collections = query
        .collections
        .iter()
        .map(|name| catalog.resolve_collection(name).to_owned())
        .collect();
    to_stac_search(query)
        .normalize_datetimes()
        .map_err(error)?
        .valid()
        .map_err(error)
}

/// Stream rows with bounded decoding; the visitor returns false to stop.
pub(crate) fn visit_items(
    path: &std::path::Path,
    search: Option<&stac::api::Search>,
    cancelled: &AtomicBool,
    mut visitor: impl FnMut(stac::Item) -> Result<bool, SuperSTACError>,
) -> Result<(), SuperSTACError> {
    let file = File::open(path).map_err(|e| error(format!("{}: {e}", path.display())))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file).map_err(error)?;
    let groups: Vec<_> = builder
        .metadata()
        .row_groups()
        .iter()
        .enumerate()
        .filter(|(_, group)| search.is_none_or(|search| pruning::may_match(group, search)))
        .map(|(i, _)| i)
        .collect();
    tracing::debug!(path = %path.display(), row_groups_total = builder.metadata().num_row_groups(), row_groups_read = groups.len(), "GeoParquet scan");
    let metadata = builder
        .geoparquet_metadata()
        .transpose()
        .map_err(error)?
        .ok_or_else(|| error("file has no GeoParquet metadata"))?;
    let schema = builder
        .geoarrow_schema(&metadata, true, Default::default())
        .map_err(error)?;
    let reader = builder
        .with_row_groups(groups)
        .with_batch_size(1024)
        .build()
        .map_err(error)?;
    let reader = GeoParquetRecordBatchReader::try_new(reader, schema).map_err(error)?;
    for batch in reader {
        if cancelled.load(Ordering::Relaxed) {
            return Err(error("snapshot search cancelled"));
        }
        let batch = batch.map_err(error)?;
        let schema = batch.schema();
        let items = stac::geoarrow::from_record_batch_reader(RecordBatchIterator::new(
            std::iter::once(Ok(batch)),
            schema,
        ))
        .map_err(error)?
        .items;
        for item in items {
            if !visitor(item)? {
                return Ok(());
            }
        }
    }
    Ok(())
}

struct CancelOnDrop(Arc<AtomicBool>);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

fn error(message: impl std::fmt::Display) -> SuperSTACError {
    SuperSTACError::SearchFailed(format!("GeoParquet: {message}"))
}
