//! Searches local STAC GeoParquet files or managed datasets with coverage manifests.
//!
//! Files retain source-local collection and asset names. Searches scan bounded
//! batches on Tokio's blocking pool; there is no spatial index or predicate
//! pushdown yet. Each query reopens the file, so publish replacements atomically.
pub mod auto;
mod budget;
pub mod ingest;
pub mod manifest;
pub mod progress;

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
        })
    }
    /// Opens one committed dataset generation; readers retain it during refreshes.
    pub fn from_dataset(root: impl AsRef<std::path::Path>) -> Result<Self, SuperSTACError> {
        let root = root.as_ref();
        let manifest = manifest::DatasetManifest::read(root)?;
        if manifest.catalogs.is_empty() {
            return Err(error("dataset has no completed snapshots"));
        }
        Ok(Self {
            paths: HashMap::new(),
            snapshots: manifest.catalogs,
            root: Some(root.to_owned()),
        })
    }

    pub fn catalog_ids(&self) -> impl Iterator<Item = &String> {
        self.paths.keys().chain(self.snapshots.keys())
    }

    fn select_snapshot(
        &self,
        catalog: &Catalog,
        search: &stac::api::Search,
        max_age: Option<std::time::Duration>,
    ) -> Option<&manifest::Snapshot> {
        self.snapshots
            .get(&catalog.id)?
            .values()
            .filter(|snapshot| {
                snapshot.source_url == catalog.url
                    && snapshot.ensure_coverage(search).is_ok()
                    && max_age.is_none_or(|max| {
                        chrono::Utc::now()
                            .signed_duration_since(snapshot.started_at)
                            .to_std()
                            .ok()
                            .is_some_and(|age| age <= max)
                    })
            })
            .max_by_key(|snapshot| (snapshot.started_at, &snapshot.name))
    }

    /// A single complete, fresh scope must cover the whole query. Scope unions
    /// are deliberately not inferred from overlapping bounding boxes/intervals.
    pub fn can_answer(
        &self,
        catalog: &Catalog,
        query: SearchQuery,
        max_age: std::time::Duration,
    ) -> Result<bool, SuperSTACError> {
        let search = prepare_query(catalog, query)?;
        Ok(self
            .select_snapshot(catalog, &search, Some(max_age))
            .is_some())
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
        let paths = if let Some(root) = &self.root {
            let snapshot = self.select_snapshot(catalog, &search, max_age).ok_or_else(|| error(format!(
                "query exceeds recorded coverage, source differs, or snapshot is stale for catalog '{}'; narrow the query or ingest another scope", catalog.id
            )))?;
            let mut paths = Vec::new();
            for file in &snapshot.files {
                if file.may_match(&search)? {
                    paths.push(manifest::resolve_file(root, &file.path)?);
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
        let cancelled = Arc::new(AtomicBool::new(false));
        let _cancel_on_drop = CancelOnDrop(cancelled.clone());
        tokio::task::spawn_blocking(move || {
            let mut results = Vec::new();
            for path in paths {
                let remaining = cap - results.len();
                if remaining == 0 || cancelled.load(Ordering::Relaxed) {
                    break;
                }
                results.extend(scan(
                    path,
                    catalog.clone(),
                    search.clone(),
                    options,
                    remaining,
                    cancelled.clone(),
                )?);
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
    if query
        .sortby
        .as_ref()
        .is_some_and(|fields| !fields.is_empty())
    {
        return Err(error(
            "sortby is not supported by the GeoParquet backend yet",
        ));
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

fn scan(
    path: PathBuf,
    catalog: Catalog,
    search: stac::api::Search,
    options: BackendSearchOptions,
    cap: usize,
    cancelled: Arc<AtomicBool>,
) -> Result<Vec<SearchItem>, SuperSTACError> {
    let file = File::open(&path).map_err(|e| error(format!("{}: {e}", path.display())))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file).map_err(error)?;
    let metadata = builder
        .geoparquet_metadata()
        .transpose()
        .map_err(error)?
        .ok_or_else(|| error("file has no GeoParquet metadata"))?;
    let schema = builder
        .geoarrow_schema(&metadata, true, Default::default())
        .map_err(error)?;
    let reader = builder.with_batch_size(1024).build().map_err(error)?;
    let mut reader = GeoParquetRecordBatchReader::try_new(reader, schema).map_err(error)?;
    let mut results = Vec::new();
    while !cancelled.load(Ordering::Relaxed) {
        let Some(batch) = reader.next() else { break };
        let batch = batch.map_err(error)?;
        let schema = batch.schema();
        let items = stac::geoarrow::from_record_batch_reader(RecordBatchIterator::new(
            std::iter::once(Ok(batch)),
            schema,
        ))
        .map_err(error)?
        .items;
        for mut item in items {
            if cancelled.load(Ordering::Relaxed) {
                return Err(error("snapshot search cancelled"));
            }
            if search.matches(&item).map_err(error)? {
                if options.unify_response {
                    unifier::unify_item(&mut item, &catalog);
                }
                results.push(SearchItem {
                    catalog_id: catalog.id.clone(),
                    seen_in: vec![catalog.id.clone()],
                    item,
                });
                if results.len() == cap {
                    return Ok(results);
                }
            }
        }
    }
    Ok(results)
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
