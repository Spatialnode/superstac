use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use crate::storages::factory::{StorageBackend};
use crate::errors::{StorageError, SuperSTACError};
use crate::models::catalog::CatalogUpdate;
use crate::models::provider::{CatalogProviderFilters, CatalogProviderUpdate};
use crate::models::settings::{Settings, SettingsUpdate};
use crate::models::{
    catalog::{Catalog, CatalogFilters},
    provider::CatalogProvider,
};

// todo - should state be shared and come from a single source, or be different depending on backend ?

// Memory backend state.
#[derive(Default, Debug)]
pub struct MemoryStorage {
    catalogs: HashMap<String, Catalog>,
    providers: HashMap<String, CatalogProvider>,
    settings: RwLock<Settings>,
}

impl MemoryStorage {
    /// Initialize the in-memory database.
    pub fn init() -> Self {
        Self {
            catalogs: HashMap::new(),
            providers: HashMap::new(),
            settings: RwLock::new(Settings::default()),
        }
    }

    fn set_catalog_provider(
        &mut self,
        catalog_id: &str,
        new_provider_id: Option<&str>,
    ) -> Result<(), StorageError> {
        let catalog = self
            .catalogs
            .get_mut(catalog_id)
            .ok_or(StorageError::CatalogDoesNotExist(catalog_id.to_string()))?;

        let old_provider = &catalog.provider;

        if let Some(old_provider) = &old_provider {
            if Some(old_provider.as_str()) == new_provider_id {
                return Ok(());
            }
        }

        // Detach from old provider
        if let Some(old) = &old_provider {
            if let Some(old_provider) = self.providers.get_mut(old) {
                old_provider.remove_catalog(catalog_id);
            }
        }

        // Attach to new provider
        if let Some(new_id) = new_provider_id {
            let new_provider = self
                .providers
                .get_mut(new_id)
                .ok_or_else(|| {
                    StorageError::CatalogProviderDoesNotExist(new_id.to_string())
                })?;

            new_provider.add_catalog(catalog_id);
            catalog.set_provider(new_id);
        }

        Ok(())
    }
}

impl StorageBackend for MemoryStorage {
    fn get_settings(&self) -> Settings {
        self.settings.read().unwrap().clone()
    }

    fn update_settings(&self, update: SettingsUpdate) {
        let mut settings = self.settings.write().unwrap();

        if let Some(health_check_strategy) = update.health_check_strategy {
            settings.health_check_strategy = health_check_strategy;
        }

        if let Some(healthy_status_code_range) = update.healthy_status_code_range {
            settings.healthy_status_code_range = healthy_status_code_range;
        }

        if let Some(auto_fix_duplicate_catalog_id) = update.auto_fix_duplicate_catalog_id {
            settings.auto_fix_duplicate_catalog_id = auto_fix_duplicate_catalog_id;
        }

        if let Some(auto_fix_duplicate_provider_id) = update.auto_fix_duplicate_provider_id {
            settings.auto_fix_duplicate_provider_id = auto_fix_duplicate_provider_id;
        }
        if let Some(logging_enabled) = update.logging_enabled {
            settings.logging_enabled = logging_enabled;
        }

        if let Some(log_level) = update.log_level {
            settings.log_level = log_level;
        }
        if let Some(search_healthy_catalogs_only) = update.search_healthy_catalogs_only {
            settings.search_healthy_catalogs_only = Some(search_healthy_catalogs_only);
        }
        if let Some(deduplicate_items) = update.deduplicate_items {
            settings.deduplicate_items = Some(deduplicate_items);
        }
        if let Some(unify_response) = update.unify_response {
            settings.unify_response = Some(unify_response);
        }
        if let Some(max_concurrent_catalogs) = update.max_concurrent_catalogs {
            settings.max_concurrent_catalogs = Some(max_concurrent_catalogs);
        }
        if let Some(per_catalog_timeout_seconds) = update.per_catalog_timeout_seconds {
            settings.per_catalog_timeout_seconds = Some(per_catalog_timeout_seconds);
        }
        if let Some(max_retry_attempts) = update.max_retry_attempts {
            settings.max_retry_attempts = Some(max_retry_attempts);
        }
        if let Some(retry_initial_backoff_ms) = update.retry_initial_backoff_ms {
            settings.retry_initial_backoff_ms = Some(retry_initial_backoff_ms);
        }
        if let Some(retry_max_backoff_ms) = update.retry_max_backoff_ms {
            settings.retry_max_backoff_ms = Some(retry_max_backoff_ms);
        }
        if let Some(max_items_per_catalog) = update.max_items_per_catalog {
            settings.max_items_per_catalog = Some(max_items_per_catalog);
        }
    }

    fn create_provider(
        &mut self,
        mut provider: CatalogProvider,
    ) -> Result<CatalogProvider, SuperSTACError> {
        if self.providers.contains_key(&provider.id) {
            if self.get_settings().auto_fix_duplicate_provider_id {
                let new_provider_id: String =
                    format!("{}-{}", &provider.id, self.providers.len() + 1);
                tracing::warn!(
                    old_id = %provider.id,
                    new_id = %new_provider_id,
                    "duplicate provider id auto-fixed"
                );
                provider.set_id(new_provider_id);
            } else {
                return Err(
                    StorageError::CatalogProviderIdAlreadyExist(provider.id.to_owned()).into(),
                );
            }
        }

        // Validate catalog references
        if let Some(catalog_ids) = &provider.catalog_ids {
            for catalog_id in catalog_ids {
                if !self.catalogs.contains_key(catalog_id) {
                    return Err(StorageError::CatalogDoesNotExist(catalog_id.to_owned()).into());
                } else {
                    // Associate the provider to the catalogs
                    for catalog_id in catalog_ids {
                        match self.catalogs.get_mut(catalog_id) {
                            Some(catalog) => {
                                if let Some(existing_provider) = &catalog.provider {
                                    return Err(StorageError::CatalogAlreadyHasProvider(format!(
                                        "Catalog '{}' is already linked to provider '{}'",
                                        catalog_id, existing_provider
                                    ))
                                    .into());
                                } else {
                                    // Attach the new provider
                                    catalog.set_provider(&provider.id)
                                }
                            }
                            None => (),
                        }
                    }
                }
            }
        }

        self.providers.insert(provider.id.clone(), provider.clone());
        Ok(provider)
    }

    fn update_provider(
        &mut self,
        id: &str,
        update: CatalogProviderUpdate,
    ) -> Result<CatalogProvider, SuperSTACError> {
       
        let matched_provider = self
            .providers
            .get_mut(id)
            .ok_or_else(|| StorageError::CatalogProviderDoesNotExist(id.to_owned()))?;

        if let Some(catalog_ids) = &update.catalog_ids {
            for catalog_id in catalog_ids {
                if !self.catalogs.contains_key(catalog_id) {
                    return Err(
                        StorageError::CatalogDoesNotExist(catalog_id.to_owned()).into(),
                    );
                }
            }
        }

        matched_provider.update(
            update.name,
            update.website_url,
            update.logo_url,
            update.description,
            update.stac_version,
            update.catalog_ids,
        )?;

        Ok(matched_provider.clone())
    }

    fn get_provider(&self, id: &str) -> Result<&CatalogProvider, SuperSTACError> {
        let provider = self.providers.get(id);
        match provider {
            Some(provider) => Ok(provider),
            None => Err(StorageError::CatalogProviderDoesNotExist(id.to_owned()))?,
        }
    }

    fn delete_provider(&mut self, id: &str) -> Result<(), SuperSTACError> {
        match self.providers.remove(id) {
            Some(_) => {
                // Detach the provider from any associated catalog.
                for catalog in self.catalogs.values_mut() {
                    if catalog.provider.is_some() {
                        catalog.provider = None;
                    }
                }
                Ok(())
            }
            None => Err(StorageError::CatalogProviderDoesNotExist(id.to_owned()))?,
        }
    }

    fn delete_providers(&mut self, ids: Vec<&str>) -> Result<(), SuperSTACError> {
        let mut not_found = Vec::new();

        for id in ids {
            if self.delete_provider(id).is_err() {
                not_found.push(id.to_string());
            }
        }

        if not_found.is_empty() {
            Ok(())
        } else {
            Err(StorageError::ProvidersNotFound(not_found))?
        }
    }

    fn list_providers(
        &self,
        filters: Option<CatalogProviderFilters>,
    ) -> Result<Vec<CatalogProvider>, SuperSTACError> {
        let mut providers: Vec<CatalogProvider> = self.providers.values().cloned().collect();

        // Apply filters.
        if let Some(filters) = filters {
            if let Some(id_val) = filters.id {
                providers.retain(|c| c.id.to_lowercase() == id_val.to_lowercase());
                return Ok(providers);
            }

            let name_filter = filters.name.as_ref();
            let description_filter = filters.description.as_ref();
            let catalog_id_filter = filters.catalog_id.as_ref();
            let stac_version_filter = filters.stac_version.as_ref();
            let created_after_filter = filters.created_after.as_ref();
            let created_before_filter = filters.created_before.as_ref();
            let updated_after_filter = filters.updated_after.as_ref();
            let updated_before_filter = filters.updated_before.as_ref();

            providers.retain(|provider| {
                let name_match = match name_filter {
                    Some(name_filter_ref) => provider
                        .name
                        .as_ref()
                        .map(|name_ref| {
                            name_ref
                                .to_lowercase()
                                .contains(name_filter_ref.to_lowercase().as_str())
                        })
                        .unwrap_or(false),
                    None => true,
                };

                let catalog_id_match = match catalog_id_filter {
                    Some(catalog_id_filter_ref) => provider
                        .catalog_ids
                        .as_ref()
                        .map(|catalog_id_ref| {
                            catalog_id_ref
                                .iter()
                                .any(|id| id.to_lowercase() == catalog_id_filter_ref.as_str())
                        })
                        .unwrap_or(false),
                    None => true,
                };

                let stac_version_match = match stac_version_filter {
                    Some(stac_version_filter_ref) => provider
                        .stac_version
                        .as_ref()
                        .map(|stac_version_ref| {
                            stac_version_ref
                                .to_lowercase()
                                .contains(stac_version_filter_ref.to_lowercase().as_str())
                        })
                        .unwrap_or(false),
                    None => true,
                };

                let description_match = match description_filter {
                    Some(d_filter_ref) => provider
                        .description
                        .as_ref()
                        .map(|desc_ref| {
                            desc_ref
                                .to_lowercase()
                                .contains(d_filter_ref.to_lowercase().as_str())
                        })
                        .unwrap_or(false),
                    None => true,
                };

                let created_at_match = match (created_after_filter, created_before_filter) {
                    (Some(after), Some(before)) => provider
                        .created_at
                        .map(|c| c >= *after && c <= *before)
                        .unwrap_or(false),
                    (Some(after), None) => {
                        provider.created_at.map(|c| c >= *after).unwrap_or(false)
                    }
                    (None, Some(before)) => {
                        provider.created_at.map(|c| c <= *before).unwrap_or(false)
                    }
                    (None, None) => true,
                };

                let updated_at_match = match (updated_after_filter, updated_before_filter) {
                    (Some(after), Some(before)) => provider
                        .updated_at
                        .map(|u| u >= *after && u <= *before)
                        .unwrap_or(false),
                    (Some(after), None) => {
                        provider.updated_at.map(|u| u >= *after).unwrap_or(false)
                    }
                    (None, Some(before)) => {
                        provider.updated_at.map(|u| u <= *before).unwrap_or(false)
                    }
                    (None, None) => true,
                };

                // Priority: Stac Vesion -> created_at -> then updated_at -> then the strings.
                stac_version_match
                    && created_at_match
                    && updated_at_match
                    && name_match
                    && description_match
                    && catalog_id_match
            });
        }

        Ok(providers)
    }

    /// **** Catalog functions ****

    fn get_catalog(&self, id: &str) -> Result<&Catalog, SuperSTACError> {
        let catalog = self.catalogs.get(id);
        match catalog {
            Some(catalog) => Ok(catalog),
            None => Err(StorageError::CatalogDoesNotExist(id.to_owned()))?,
        }
    }

    fn update_catalog(
        &mut self,
        id: &str,
        update: CatalogUpdate,
    ) -> Result<Catalog, SuperSTACError> {
        let catalog_id = self
            .catalogs
            .get(id)
            .map(|c| c.id.clone())
            .ok_or_else(|| StorageError::CatalogDoesNotExist(id.to_owned()))?
            .clone();

        // Update provider relationship
        self.set_catalog_provider(&catalog_id, update.provider.as_deref())?;

        let matched_catalog = self.catalogs.get_mut(id).unwrap();
        matched_catalog.update(
            update.description,
            update.url,
            update.title,
            update.settings,
        )?;
        Ok(matched_catalog.clone())
    }

    fn create_catalog(
        &mut self,
        mut catalog: Catalog,
        provider_id: Option<&str>,
    ) -> Result<Catalog, SuperSTACError> {
        // Merge the pkey with the catalog id if the catalog with the same id already exists. That way it's unique.
        if self.catalogs.contains_key(&catalog.id) {
            if self.get_settings().auto_fix_duplicate_catalog_id {
                let new_catalog_id = format!("{}-{}", catalog.id, self.catalogs.len() + 1);
                tracing::warn!(
                    old_id = %catalog.id,
                    new_id = %new_catalog_id,
                    "duplicate catalog id auto-fixed"
                );
                catalog.set_id(new_catalog_id);
            } else {
                return Err(StorageError::CatalogIdAlreadyExist(catalog.id.to_owned()).into());
            }
        }
        let catalog_id = catalog.id.clone();

        self.catalogs.insert(catalog_id, catalog.clone());

        self.set_catalog_provider(&catalog.id, provider_id)?;

        Ok(catalog)
    }

    fn list_catalogs(
        &self,
        filters: Option<CatalogFilters>,
    ) -> Result<Vec<Catalog>, SuperSTACError> {
        let mut catalogs: Vec<Catalog> = self.catalogs.values().cloned().collect();

        // Apply filters.
        if let Some(filters) = filters {
            if let Some(id_val) = filters.id {
                catalogs.retain(|c| c.id.to_lowercase() == id_val.to_lowercase());
                return Ok(catalogs);
            }

            let provider_filter = filters.provider.as_ref();
            let title_filter = filters.title.as_ref();
            let description_filter = filters.description.as_ref();
            let created_after_filter = filters.created_after.as_ref();
            let created_before_filter = filters.created_before.as_ref();
            let updated_after_filter = filters.updated_after.as_ref();
            let updated_before_filter = filters.updated_before.as_ref();

            catalogs.retain(|catalog| {
                let health_status_match = match filters.available {
                    Some(is_available) => catalog.health_status.available == is_available,
                    None => true,
                };

                let provider_match = match provider_filter {
                    Some(filter) => {
                        let filter_lc = filter.to_lowercase();
                        catalog.provider.as_ref().map_or(false, |provider_id| {
                            self.get_provider(provider_id).ok().map_or(false, |p| {
                                let name_matches = p
                                    .name
                                    .as_ref()
                                    .map_or(false, |n| n.to_lowercase().contains(&filter_lc));

                                let desc_matches = p
                                    .description
                                    .as_ref()
                                    .map_or(false, |d| d.to_lowercase().contains(&filter_lc));

                                name_matches || desc_matches
                            })
                        })
                    }
                    None => true,
                };

                let title_match = match title_filter {
                    Some(t_filter_ref) => catalog
                        .title
                        .as_ref()
                        .map(|title_ref| {
                            title_ref
                                .to_lowercase()
                                .contains(t_filter_ref.to_lowercase().as_str())
                        })
                        .unwrap_or(false),
                    None => true,
                };

                let description_match = match description_filter {
                    Some(d_filter_ref) => catalog
                        .description
                        .as_ref()
                        .map(|desc_ref| {
                            desc_ref
                                .to_lowercase()
                                .contains(d_filter_ref.to_lowercase().as_str())
                        })
                        .unwrap_or(false),
                    None => true,
                };

                let created_at_match = match (created_after_filter, created_before_filter) {
                    (Some(after), Some(before)) => catalog
                        .created_at
                        .map(|c| c >= *after && c <= *before)
                        .unwrap_or(false),
                    (Some(after), None) => catalog.created_at.map(|c| c >= *after).unwrap_or(false),
                    (None, Some(before)) => {
                        catalog.created_at.map(|c| c <= *before).unwrap_or(false)
                    }
                    (None, None) => true,
                };

                let updated_at_match = match (updated_after_filter, updated_before_filter) {
                    (Some(after), Some(before)) => catalog
                        .updated_at
                        .map(|u| u >= *after && u <= *before)
                        .unwrap_or(false),
                    (Some(after), None) => catalog.updated_at.map(|u| u >= *after).unwrap_or(false),
                    (None, Some(before)) => {
                        catalog.updated_at.map(|u| u <= *before).unwrap_or(false)
                    }
                    (None, None) => true,
                };

                // Priority: Health status first -> then created_at -> then updated_at -> then the strings.
                health_status_match
                    && created_at_match
                    && updated_at_match
                    && title_match
                    && description_match
                    && provider_match
            });
        }

        Ok(catalogs)
    }

    fn delete_catalog(&mut self, id: &str) -> Result<(), SuperSTACError> {
        match self.catalogs.remove(id) {
            Some(_) => {
                // Detach the catalog id from any associated providers.
                for provider in self.providers.values_mut() {
                    if let Some(catalogs) = &mut provider.catalog_ids {
                        catalogs.retain(|catalog_id| catalog_id != id);
                    }
                }
                Ok(())
            }
            None => Err(StorageError::CatalogDoesNotExist(id.to_owned()))?,
        }
    }

    fn delete_catalogs(&mut self, ids: Vec<&str>) -> Result<(), SuperSTACError> {
        let mut not_found = Vec::new();

        for id in ids {
            if self.delete_catalog(id).is_err() {
                not_found.push(id.to_string());
            }
        }

        if not_found.is_empty() {
            Ok(())
        } else {
            Err(StorageError::CatalogsNotFound(not_found))?
        }
    }

    fn update_health(&mut self, id: &str, healthy: bool) -> Result<(), SuperSTACError> {
        match self.catalogs.get_mut(id) {
            Some(catalog) => {
                catalog.health_status.available = healthy;
                Ok(())
            }
            None => Err(StorageError::CatalogDoesNotExist(id.to_owned()))?,
        }
    }

    fn update_supported_collections(
        &mut self,
        id: &str,
        collections: Option<HashSet<String>>,
    ) -> Result<(), SuperSTACError> {
        match self.catalogs.get_mut(id) {
            Some(catalog) => {
                catalog.supported_collections = collections;
                Ok(())
            }
            None => Err(StorageError::CatalogDoesNotExist(id.to_owned()))?,
        }
    }
}
