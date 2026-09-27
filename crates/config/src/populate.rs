use superstac_core::{
    errors::SuperSTACError,
    models::{catalog::Catalog, provider::CatalogProvider, settings::SettingsUpdate},
    storages::factory::StorageBackend,
};

use crate::config::SuperStacConfig;

/// Seed a backend with the providers, catalogs, and settings parsed from a
/// [`SuperStacConfig`]. Generic over the backend trait so future backends work without changes here.
pub fn populate_backend_from_config(
    backend: &mut dyn StorageBackend,
    config: SuperStacConfig,
) -> Result<(), SuperSTACError> {
    tracing::debug!(
        providers = config.providers.len(),
        catalogs = config.catalogs.len(),
        "seeding backend from config"
    );

    backend.update_settings(SettingsUpdate {
        health_check_strategy: Some(config.settings.health_check_strategy),
        healthy_status_code_range: Some(config.settings.healthy_status_code_range),
        auto_fix_duplicate_catalog_id: Some(config.settings.auto_fix_duplicate_catalog_id),
        auto_fix_duplicate_provider_id: Some(config.settings.auto_fix_duplicate_provider_id),
        logging_enabled: Some(config.settings.logging_enabled),
        log_level: Some(config.settings.log_level),
        search_healthy_catalogs_only: config.settings.search_healthy_catalogs_only,
        deduplicate_items: config.settings.deduplicate_items,
        unify_response: config.settings.unify_response,
        max_concurrent_catalogs: config.settings.max_concurrent_catalogs,
        per_catalog_timeout_seconds: config.settings.per_catalog_timeout_seconds,
        max_retry_attempts: config.settings.max_retry_attempts,
        retry_initial_backoff_ms: config.settings.retry_initial_backoff_ms,
        retry_max_backoff_ms: config.settings.retry_max_backoff_ms,
        max_items_per_catalog: config.settings.max_items_per_catalog,
        enable_background_health_monitor: config.settings.enable_background_health_monitor,
    });

    for provider_cfg in config.providers {
        let provider = CatalogProvider::try_from(provider_cfg)?;
        backend.create_provider(provider)?;
    }

    for catalog_cfg in config.catalogs {
        let provider_ref = catalog_cfg.provider.clone();
        let catalog = Catalog::try_from(catalog_cfg)?;
        backend.create_catalog(catalog, provider_ref.as_deref())?;
    }

    Ok(())
}
