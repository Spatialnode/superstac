use parking_lot::Mutex;
use reqwest::Client;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use std::collections::{HashMap, HashSet};

use crate::{
    capabilities,
    discovery::{self, CollectionAvailability},
    health::HealthManager,
    types::SharedStorage,
};
use std::time::Duration;
use superstac_core::{
    errors::SuperSTACError,
    models::catalog::{Catalog, CatalogFilters},
    storages::factory::StorageBackend,
};

use superstac_search::{
    executor::SearchExecutor,
    // TODO - Move this to settings.
    options::{FederationOptions, RetryPolicy},
    query::SearchQuery,
    response::SearchResponse,
};

/// Runtime entry point. Wraps a storage backend, runs background health
/// checks and `/collections` introspection, and exposes the federated search
/// + discovery API.
///
/// Construct, call `start()`, then use `search()` / `list_collections()` /
/// `describe_collection()`. `start()` is idempotent — `search()` will call
/// it for you if needed.
pub struct SuperSTACEngine {
    storage: SharedStorage,
    client: Client,
    health_manager: Option<HealthManager>,
    executor: SearchExecutor,
    snapshot_catalog_ids: Option<HashSet<String>>,
    auto_search: bool,
    #[cfg(feature = "geoparquet")]
    local_backend: Option<Arc<superstac_geoparquet::GeoParquetBackend>>,
    pub started: AtomicBool,
}

/// User-Agent attached to every outbound HTTP request. Set as a default
/// header on the shared reqwest client at engine construction. When
/// per-catalog headers are added later, this must be filtered out of any
/// user-supplied set so it can't be overridden.
const USER_AGENT: &'static str = concat!("superstac/", env!("CARGO_PKG_VERSION"));

impl SuperSTACEngine {
    /// Build an engine over the given storage. Construct the storage via
    /// `superstac_config::init_from_yaml` or directly via
    /// [`superstac_core::models::storage::Storage::init`].
    pub fn new(storage: Box<dyn StorageBackend + Send + Sync>) -> Self {
        Self::from_shared(Arc::new(Mutex::new(storage)))
    }

    /// Build an engine over an already-shared storage handle. Useful when a
    /// caller (e.g. the Python bindings) needs to retain mutation access to
    /// the same backend the engine reads from.
    pub fn from_shared(storage: SharedStorage) -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("failed to build reqwest client");

        Self {
            storage,
            client: client.clone(),
            health_manager: Some(HealthManager::new(client.clone())),
            executor: SearchExecutor::new(client),
            snapshot_catalog_ids: None,
            auto_search: false,
            #[cfg(feature = "geoparquet")]
            local_backend: None,
            started: AtomicBool::new(false),
        }
    }

    /// Search only the supplied local snapshots. IDs must already exist in storage.
    /// Provider health, API introspection, and cached remote collection inventories
    /// are ignored. Collection discovery is not yet available in snapshot mode.
    #[cfg(feature = "geoparquet")]
    pub fn from_geoparquet(
        storage: Box<dyn StorageBackend + Send + Sync>,
        paths: HashMap<String, std::path::PathBuf>,
    ) -> Result<Self, SuperSTACError> {
        Self::from_shared_geoparquet(Arc::new(Mutex::new(storage)), paths)
    }

    #[cfg(feature = "geoparquet")]
    pub fn from_shared_geoparquet(
        storage: SharedStorage,
        paths: HashMap<String, std::path::PathBuf>,
    ) -> Result<Self, SuperSTACError> {
        for id in paths.keys() {
            storage.lock().get_catalog(id)?;
        }
        let ids = paths.keys().cloned().collect();
        let backend = superstac_geoparquet::GeoParquetBackend::new(paths)?;
        let mut engine = Self::from_shared(storage);
        let backend = Arc::new(backend);
        engine.executor = SearchExecutor::with_backend(Box::new(backend.clone()));
        engine.local_backend = Some(backend);
        engine.health_manager = None;
        engine.snapshot_catalog_ids = Some(ids);
        Ok(engine)
    }

    /// Open completed catalog snapshots from a managed dataset. Search coverage
    /// is checked against the manifest; no provider checks or requests occur.
    #[cfg(feature = "geoparquet")]
    pub fn from_dataset(
        storage: Box<dyn StorageBackend + Send + Sync>,
        root: impl AsRef<std::path::Path>,
    ) -> Result<Self, SuperSTACError> {
        Self::from_shared_dataset(Arc::new(Mutex::new(storage)), root)
    }

    #[cfg(feature = "geoparquet")]
    pub fn from_shared_dataset(
        storage: SharedStorage,
        root: impl AsRef<std::path::Path>,
    ) -> Result<Self, SuperSTACError> {
        let backend = superstac_geoparquet::GeoParquetBackend::from_dataset(root)?;
        for id in backend.catalog_ids() {
            storage.lock().get_catalog(id)?;
        }
        let ids = backend.catalog_ids().cloned().collect();
        let mut engine = Self::from_shared(storage);
        let backend = Arc::new(backend);
        engine.executor = SearchExecutor::with_backend(Box::new(backend.clone()));
        engine.local_backend = Some(backend);
        engine.health_manager = None;
        engine.snapshot_catalog_ids = Some(ids);
        Ok(engine)
    }

    /// Prefer one fresh covering snapshot per catalog, otherwise query its API.
    /// This mode never writes datasets and skips provider monitoring/introspection.
    #[cfg(feature = "geoparquet")]
    pub fn automatic(
        storage: Box<dyn StorageBackend + Send + Sync>,
        root: impl Into<std::path::PathBuf>,
        max_snapshot_age: Duration,
    ) -> Self {
        Self::from_shared_automatic(Arc::new(Mutex::new(storage)), root, max_snapshot_age)
    }

    #[cfg(feature = "geoparquet")]
    pub fn from_shared_automatic(
        storage: SharedStorage,
        root: impl Into<std::path::PathBuf>,
        max_snapshot_age: Duration,
    ) -> Self {
        let mut engine = Self::from_shared(storage);
        let backend = superstac_geoparquet::auto::AutoBackend::new(
            root.into(),
            max_snapshot_age,
            engine.client.clone(),
        );
        engine.executor = SearchExecutor::with_backend(Box::new(backend));
        engine.health_manager = None;
        engine.auto_search = true;
        engine
    }

    async fn discovery_catalogs(&self) -> Result<Vec<Catalog>, SuperSTACError> {
        self.ensure_started().await?;
        let mut catalogs = self.storage.lock().list_catalogs(None)?;
        #[cfg(feature = "geoparquet")]
        if let Some(backend) = &self.local_backend {
            catalogs.retain(|c| {
                self.snapshot_catalog_ids
                    .as_ref()
                    .is_some_and(|ids| ids.contains(&c.id))
            });
            for catalog in &mut catalogs {
                let backend = backend.clone();
                let copy = catalog.clone();
                let collections = tokio::task::spawn_blocking(move || backend.collections(&copy))
                    .await
                    .map_err(|e| SuperSTACError::SearchFailed(e.to_string()))??;
                catalog.supported_collections = Some(collections.into_keys().collect());
            }
        }
        if self.auto_search {
            for catalog in &mut catalogs {
                catalog.supported_collections =
                    Some(capabilities::fetch_supported_collections(&self.client, catalog).await?);
            }
        }
        Ok(catalogs)
    }

    /// Run health checks against all catalogs, then introspect `/collections`
    /// on the healthy ones. Snapshot mode skips all provider access.
    /// Idempotent — safe to call multiple times.
    pub async fn start(&self) -> Result<(), SuperSTACError> {
        if self.started.load(Ordering::Relaxed) {
            return Ok(());
        }

        if self.snapshot_catalog_ids.is_some() || self.auto_search {
            self.started.store(true, Ordering::Relaxed);
            return Ok(());
        }

        if let Some(manager) = &self.health_manager {
            let catalogs = {
                let storage = self.storage.lock();
                storage.list_catalogs(None)?
            };

            manager.start(Arc::clone(&self.storage), catalogs).await?;
        }

        // Introspect each healthy catalog's /collections so we can do source
        // selection at search time. Failures here are non-fatal — affected
        // catalogs remain `supported_collections = None` (pass-through).
        // TODO - Maybe have a way to disable this by default ?
        self.introspect_capabilities().await?;

        self.started.store(true, Ordering::Relaxed);

        let catalog_count = self.storage.lock().list_catalogs(None)?.len();
        tracing::info!(catalogs = catalog_count, "engine ready");

        Ok(())
    }

    /// Cancel background health-monitor tasks. Doesn't drop the storage.
    pub async fn shutdown(&self) {
        tracing::debug!("shutting down engine");

        if let Some(manager) = &self.health_manager {
            manager.stop_all().await;
        }

        self.started.store(false, Ordering::Relaxed);
    }

    /// Run health checks against every catalog, update storage with results. Then update the `supported_collections` sets for any newly-healthy catalogs.
    async fn introspect_capabilities(&self) -> Result<(), SuperSTACError> {
        let healthy_catalogs = {
            let storage = self.storage.lock();
            storage.list_catalogs(Some(CatalogFilters {
                available: Some(true),
                ..CatalogFilters::default()
            }))?
        };

        for catalog in healthy_catalogs {
            match capabilities::fetch_supported_collections(&self.client, &catalog).await {
                Ok(set) => {
                    tracing::debug!(
                        catalog = %catalog.id,
                        collections = set.len(),
                        "introspected /collections"
                    );
                    let mut storage = self.storage.lock();
                    if let Err(e) = storage.update_supported_collections(&catalog.id, Some(set)) {
                        tracing::warn!(
                            catalog = %catalog.id,
                            error = %e,
                            "failed to persist /collections result"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        catalog = %catalog.id,
                        error = %e,
                        "/collections introspection failed; catalog will pass-through"
                    );
                }
            }
        }

        Ok(())
    }

    /// `start()` if we haven't already. Called automatically by `search()`
    /// and the discovery methods.
    pub async fn ensure_started(&self) -> Result<(), SuperSTACError> {
        if !self.started.load(Ordering::Relaxed) {
            self.start().await?;
        }

        Ok(())
    }

    /// Federated search. Applies source selection (filter catalogs that
    /// can't possibly serve the requested collections), then fans out.
    pub async fn search(&self, query: SearchQuery) -> Result<SearchResponse, SuperSTACError> {
        self.ensure_started().await?;

        let (candidate_catalogs, options) = {
            let storage = self.storage.lock();
            let settings = storage.get_settings();

            let catalogs = if let Some(ids) = &self.snapshot_catalog_ids {
                let mut catalogs = Vec::new();
                for id in ids {
                    let mut catalog = storage.get_catalog(id)?.clone();
                    // A remote inventory may be stale or differ from the snapshot.
                    catalog.supported_collections = None;
                    catalogs.push(catalog);
                }
                catalogs
            } else if self.auto_search {
                let mut catalogs = storage.list_catalogs(None)?;
                for catalog in &mut catalogs {
                    catalog.supported_collections = None;
                }
                catalogs
            } else if let Some(true) = settings.search_healthy_catalogs_only {
                storage.list_catalogs(Some(CatalogFilters {
                    available: Some(true),
                    ..CatalogFilters::default()
                }))?
            } else {
                storage.list_catalogs(None)?
            };

            let options = FederationOptions {
                deduplicate: settings.deduplicate_items.unwrap_or(true),
                unify_response: settings.unify_response.unwrap_or(true),
                max_concurrent: settings.max_concurrent_catalogs.unwrap_or(8),
                per_catalog_timeout: Duration::from_secs(
                    settings.per_catalog_timeout_seconds.unwrap_or(30),
                ),
                retry: RetryPolicy {
                    max_attempts: if self.snapshot_catalog_ids.is_some() {
                        1
                    } else {
                        settings.max_retry_attempts.unwrap_or(2)
                    },
                    initial_backoff: Duration::from_millis(
                        settings.retry_initial_backoff_ms.unwrap_or(100),
                    ),
                    max_backoff: Duration::from_millis(
                        settings.retry_max_backoff_ms.unwrap_or(2000),
                    ),
                },
                max_items_per_catalog: settings.max_items_per_catalog.unwrap_or(1000),
            };

            (catalogs, options)
        };

        // Compute unsupported collections against the full candidate set
        // before source-selection filtering.
        let unsupported =
            discovery::unsupported_collections(&candidate_catalogs, &query.collections);

        // Source selection: drop catalogs whose introspected collection set
        // doesn't overlap the requested collections. Catalogs with no
        // introspection data pass through.
        let catalogs: Vec<Catalog> = candidate_catalogs
            .into_iter()
            .filter(|c| c.supports_any_of(&query.collections))
            .collect();

        // Perform the federated search across the selected catalogs.
        let mut response = self
            .executor
            .federated_search(catalogs, query, options)
            .await?;

        response.metadata.unsupported_collections = unsupported;

        Ok(response)
    }

    /// Aggregated view: every collection ID known across healthy catalogs,
    /// with the catalogs that serve each.
    pub async fn list_collections(&self) -> Result<Vec<CollectionAvailability>, SuperSTACError> {
        let catalogs = self.discovery_catalogs().await?;
        Ok(discovery::aggregate_collections(&catalogs))
    }

    /// IDs of every catalog whose introspected `/collections` includes
    /// `collection_id`. Catalogs not yet introspected are excluded.
    pub async fn catalogs_supporting(
        &self,
        collection_id: &str,
    ) -> Result<Vec<String>, SuperSTACError> {
        let catalogs = self.discovery_catalogs().await?;
        Ok(discovery::catalogs_supporting(&catalogs, collection_id))
    }

    /// Per-catalog inventory: catalog ID -> sorted canonical collection IDs.
    /// Catalogs without introspection data are omitted.
    pub async fn collections_by_catalog(
        &self,
    ) -> Result<HashMap<String, Vec<String>>, SuperSTACError> {
        let catalogs = self.discovery_catalogs().await?;
        Ok(discovery::collections_by_catalog(&catalogs))
    }

    /// Fetch full collection metadata from a specific catalog. The
    /// `collection_id` is interpreted as canonical and resolved to the
    /// catalog's local name via `collection_aliases`. Returns `None` if the
    /// catalog returns 404 for the collection.
    pub async fn describe_collection(
        &self,
        catalog_id: &str,
        collection_id: &str,
    ) -> Result<Option<stac::Collection>, SuperSTACError> {
        self.ensure_started().await?;

        let catalog = {
            let storage = self.storage.lock();
            storage.get_catalog(catalog_id)?.clone()
        };

        #[cfg(feature = "geoparquet")]
        if let Some(backend) = &self.local_backend {
            let backend = backend.clone();
            let id = collection_id.to_owned();
            return tokio::task::spawn_blocking(move || {
                backend.collections(&catalog).map(|mut c| c.remove(&id))
            })
            .await
            .map_err(|e| SuperSTACError::SearchFailed(e.to_string()))?;
        }

        let local_id = catalog.resolve_collection(collection_id).to_string();

        let stac_client = stac_io::api::Client::with_client(self.client.clone(), &catalog.url)
            .map_err(|e| SuperSTACError::SearchFailed(format!("stac client init: {}", e)))?;

        stac_client
            .collection(&local_id)
            .await
            .map_err(|e| SuperSTACError::SearchFailed(format!("fetch collection: {}", e)))
    }
}
