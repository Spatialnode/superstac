use parking_lot::Mutex;
use reqwest::Client;
use std::{collections::HashMap, sync::Arc};
use tokio::{task::JoinHandle, time::interval};

use crate::types::SharedStorage;
use superstac_core::{errors::SuperSTACError, models::catalog::Catalog};

/// Manager for background health monitoring tasks. Responsible for starting/stopping monitors and performing health checks.
pub struct HealthManager {
    /// Catalog ID -> background task handle. Used for stopping monitors when catalogs are removed or on shutdown.
    tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    /// HTTP client for performing health check requests. Cloned into each monitor task.
    client: Client,
}


impl HealthManager {
    pub fn new(client: Client) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            client,
        }
    }

    /// Start background health monitors for every catalog in the given list that has a healthy initial status and an enabled monitor setting.
    /// Catalogs with an unhealthy initial status are skipped (no monitor is spawned), but will be included in future checks if they later become healthy.
    pub async fn start(
        &self,
        storage: SharedStorage,
        catalogs: Vec<Catalog>,
    ) -> Result<(), SuperSTACError> {

        tracing::debug!(catalogs = catalogs.len(), "starting health monitor");

        for catalog in catalogs {
            let healthy = self.check_once(Arc::clone(&storage), &catalog).await;

            if !healthy {
                continue;
            }

            tracing::debug!(
                catalog = %catalog.id,
                "initial health check passed"
            );

            let global_enable_monitor = storage
                .lock()
                .get_settings()
                .enable_background_health_monitor
                .unwrap_or(true);

            // catalog setting takes priority, fallback to global default
            let enable_monitor = catalog
                .settings
                .enable_background_health_monitor
                .unwrap_or(global_enable_monitor);

            if enable_monitor {
                tracing::debug!(
                    catalog = %catalog.id,
                    "spawning background health monitor"
                );
                // TODO - Limit the number of concurrent monitors if there are a large number of catalogs. 
                // This could be done with a semaphore or by having a fixed number of worker tasks that check the health of multiple catalogs in batches.
                self.spawn_health_monitor(Arc::clone(&storage), catalog);
            } else {
                tracing::debug!(
                    catalog = %catalog.id,
                    "background health monitor disabled"
                );
            }
        }

        Ok(())
    }

    /// Perform a single health check by sending a GET request to the catalog's health endpoint and checking if the status code is in the healthy range.
    /// Returns true if the catalog is healthy, false if it's unhealthy or if the request fails.
    async fn check_health_status(client: &Client, endpoint: &str, range: (u16, u16)) -> bool {
        match client.get(endpoint).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();

                range.0 <= status && status <= range.1
            }

            Err(_) => false,
        }
    }
    /// Update the health status of the given catalog in storage. Logs a warning if the update fails, but otherwise ignores storage errors (health status is best-effort and shouldn't cause cascading failures).
    fn update_health_status(storage: &SharedStorage, catalog_id: &str, healthy: bool) {
        let mut storage = storage.lock();

        if let Err(e) = storage.update_health(catalog_id, healthy) {
            tracing::warn!(
                catalog = %catalog_id,
                error = %e,
                "failed to persist health update"
            );
        }
    }
    /// Stop all background health monitors. Used for graceful shutdown.
    pub async fn stop_all(&self) {
        let mut tasks = self.tasks.lock();

        for (_, handle) in tasks.drain() {
            handle.abort();
        }
    }

    /// Stop the background health monitor for the given catalog, if it exists.
    pub async fn stop(&self, id: &str) {
        if let Some(handle) = self.tasks.lock().remove(id) {
            handle.abort();
        }
    }

    /// Perform a single health check for the given catalog and update its status in storage. Returns the health status.
    pub async fn check_once(&self, storage: SharedStorage, catalog: &Catalog) -> bool {
        let endpoint = &catalog.health_status.endpoint;

        let range = catalog.settings.healthy_status_code_range;

        let healthy = Self::check_health_status(&self.client, endpoint, range).await;

        tracing::debug!(catalog = %catalog.id, healthy, "health check");

        Self::update_health_status(&storage, &catalog.id, healthy);

        healthy
    }

    /// Spawn a background task that periodically checks the health status of the given catalog and updates it in storage.
    /// If a task for this catalog already exists, does nothing.
    pub fn spawn_health_monitor(&self, storage: SharedStorage, catalog: Catalog) {
        let id = catalog.id.clone();

        if self.tasks.lock().contains_key(&id) {
            return;
        }

        let endpoint = catalog.health_status.endpoint.clone();

        let range = catalog.settings.healthy_status_code_range;

        let duration = catalog.settings.health_check_strategy.as_duration();

        let client = self.client.clone();

        let task_id = id.clone();

        let handle = tokio::spawn(async move {
            let mut ticker = interval(duration);

            loop {
                ticker.tick().await;

                let healthy = Self::check_health_status(&client, &endpoint, range).await;

                tracing::debug!(catalog = %task_id, healthy, "health tick");

                Self::update_health_status(&storage, &task_id, healthy);
            }
        });

        tracing::debug!(
            catalog = %id,
            interval_secs = duration.as_secs(),
            "spawned health monitor"
        );

        self.tasks.lock().insert(id, handle);
    }
}
