use parking_lot::Mutex;
use reqwest::Client;
use std::{collections::HashMap, sync::Arc};
use tokio::{task::JoinHandle, time::interval};

use crate::types::SharedStorage;
use superstac_core::{errors::SuperSTACError, models::catalog::Catalog};

pub struct HealthManager {
    tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    client: Client,
}

impl HealthManager {
    pub fn new(client: Client) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            client,
        }
    }

    pub async fn start(
        &self,
        storage: SharedStorage,
        catalogs: Vec<Catalog>,
    ) -> Result<(), SuperSTACError> {
        tracing::debug!(catalogs = catalogs.len(), "starting health monitor");

        for catalog in catalogs {
            let healthy = self.check_once(Arc::clone(&storage), &catalog).await;

            if healthy {
                // TODO, check settings to decide whether to spawn monitor or not
                self.spawn_monitor(Arc::clone(&storage), catalog);
            }
        }

        Ok(())
    }

    pub async fn stop_all(&self) {
        let mut tasks = self.tasks.lock();

        for (_, handle) in tasks.drain() {
            handle.abort();
        }
    }

    pub async fn stop(&self, id: &str) {
        if let Some(handle) = self.tasks.lock().remove(id) {
            handle.abort();
        }
    }

    pub async fn check_once(&self, storage: SharedStorage, catalog: &Catalog) -> bool {
        let endpoint = &catalog.health_status.endpoint;

        let range = catalog.settings.healthy_status_code_range;

        let healthy = match self.client.get(endpoint).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();

                range.0 <= status && status <= range.1
            }

            Err(_) => false,
        };

        tracing::debug!(catalog = %catalog.id, healthy, "health check");

        {
            let mut storage = storage.lock();

            if let Err(e) = storage.update_health(&catalog.id, healthy) {
                tracing::warn!(catalog = %catalog.id, error = %e, "failed to persist health update");
            }
        }

        healthy
    }

    pub fn spawn_monitor(&self, storage: SharedStorage, catalog: Catalog) {
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

                let healthy = match client.get(&endpoint).send().await {
                    Ok(resp) => {
                        let status = resp.status().as_u16();

                        range.0 <= status && status <= range.1
                    }

                    Err(_) => false,
                };

                tracing::debug!(catalog = %task_id, healthy, "health tick");

                {
                    let mut storage = storage.lock();

                    if let Err(e) = storage.update_health(&task_id, healthy) {
                        tracing::warn!(
                            catalog = %task_id,
                            error = %e,
                            "failed to persist health update"
                        );
                    }
                }
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
