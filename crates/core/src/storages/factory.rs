use std::collections::HashSet;

use crate::{
    errors::SuperSTACError,
    models::{
        catalog::{Catalog, CatalogFilters, CatalogUpdate},
        provider::{CatalogProvider, CatalogProviderFilters, CatalogProviderUpdate},
        settings::{Settings, SettingsUpdate},
    },
};

/// Storage trait implemented by each backend (memory, sqlite, postgres, ...).
///
/// All federated-search state — catalogs, providers, settings — lives behind
/// this trait so the rest of the codebase doesn't know or care which backend
/// is in use.
pub trait StorageBackend: Send + Sync {
    /// Apply partial settings update. `None` fields are left alone.
    fn update_settings(&self, update: SettingsUpdate);

    /// Current settings snapshot.
    fn get_settings(&self) -> Settings;

    /// Get a provider by id.
    fn get_provider(&self, id: &str) -> Result<&CatalogProvider, SuperSTACError>;

    /// Insert a provider. Honors `auto_fix_duplicate_provider_id` if id collides.
    fn create_provider(
        &mut self,
        provider: CatalogProvider,
    ) -> Result<CatalogProvider, SuperSTACError>;

    /// Delete a provider by id.
    fn delete_provider(&mut self, id: &str) -> Result<(), SuperSTACError>;

    /// Delete multiple providers. Errors list ids that weren't found.
    fn delete_providers(&mut self, ids: Vec<&str>) -> Result<(), SuperSTACError>;

    /// List providers, optionally filtered. See [`CatalogProviderFilters`].
    fn list_providers(
        &self,
        filters: Option<CatalogProviderFilters>,
    ) -> Result<Vec<CatalogProvider>, SuperSTACError>;

    /// Apply a partial provider update by id. `None` fields are left alone.
    fn update_provider(
        &mut self,
        id: &str,
        update: CatalogProviderUpdate,
    ) -> Result<CatalogProvider, SuperSTACError>;

    /// Get a catalog by id.
    fn get_catalog(&self, id: &str) -> Result<&Catalog, SuperSTACError>;

    /// Apply a partial catalog update by id. `None` fields are left alone.
    fn update_catalog(
        &mut self,
        id: &str,
        update: CatalogUpdate,
    ) -> Result<Catalog, SuperSTACError>;

    /// Insert a catalog, optionally linked to a provider.
    /// Honors `auto_fix_duplicate_catalog_id` if id collides.
    fn create_catalog(
        &mut self,
        catalog: Catalog,
        provider: Option<&str>,
    ) -> Result<Catalog, SuperSTACError>;

    /// List catalogs, optionally filtered. See [`CatalogFilters`].
    fn list_catalogs(
        &self,
        filters: Option<CatalogFilters>,
    ) -> Result<Vec<Catalog>, SuperSTACError>;

    /// Delete a catalog by id.
    fn delete_catalog(&mut self, id: &str) -> Result<(), SuperSTACError>;

    /// Delete multiple catalogs. Errors list ids that weren't found.
    fn delete_catalogs(&mut self, ids: Vec<&str>) -> Result<(), SuperSTACError>;

    /// Mark a catalog healthy or unhealthy. Called by the health monitor.
    fn update_health(&mut self, id: &str, healthy: bool) -> Result<(), SuperSTACError>;

    /// Set the canonical collection IDs this catalog supports.
    /// `None` clears prior introspection. Called after `/collections` fetch.
    fn update_supported_collections(
        &mut self,
        id: &str,
        collections: Option<HashSet<String>>,
    ) -> Result<(), SuperSTACError>;
}
