use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{SuperSTACError, ValidationError},
    utils::{get_datetime_utc, parse_url, validate_identifier},
};

/// A vendor/organization that operates one or more STAC catalogs. Mostly
/// metadata (name, logo, website) — the actual API endpoints live on
/// [`super::catalog::Catalog`], linked back via `catalog_ids`.
#[derive(Clone, Debug, Serialize, Deserialize, Default,PartialEq)]
pub struct CatalogProvider {
    /// A unique id for the provider. E.g microsoft, google, element84 e.t.c.
    pub id: String,

    /// The name of the provider. E.g Microsoft, Google, Element 84 e.t.c
    pub name: Option<String>,

    /// Detailed description of the provider.
    pub description: Option<String>,

    /// URL to the provider logo. Could be a URL or a file path.
    pub logo_url: Option<String>,

    /// The STAC version the provider is conforming to.
    // pub stac_version: Option<String>,

    /// The URL to the provider website/public page.
    pub website_url: Option<String>,

    /// The related catalogs to this provider
    pub catalog_ids: Option<Vec<String>>,

    /// The date created. This is automatically populated after creation.
    pub created_at: Option<DateTime<Utc>>,
    /// The date updated. The is automatically populated after an update activity.
    pub updated_at: Option<DateTime<Utc>>,
}

impl CatalogProvider {
    /// Validate and construct a provider. Errors if the id contains
    /// non-ASCII characters or either URL doesn't parse.
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        website_url: Option<String>,
        logo_url: Option<String>,
        // stac_version: Option<String>,
        catalog_ids: Option<Vec<String>>,
    ) -> Result<Self, SuperSTACError> {
        validate_identifier(&id)?;

        let valid_website = if let Some(w) = website_url {
            parse_url(w.as_str())
                .map_err(|e| ValidationError::InvalidUrl(format!("Invalid website URL: {}", e)))?;
            Some(w.to_string())
        } else {
            None
        };

        // Validate logo_url if provided
        let valid_logo = if let Some(l) = logo_url {
            parse_url(l.as_str())
                .map_err(|e| ValidationError::InvalidUrl(format!("Invalid logo URL: {}", e)))?;
            Some(l.to_string())
        } else {
            None
        };

        // How do validate the catalogs ?, access db.catalogs here ? pass db as a parameter ?

        Ok(Self {
            id,
            name,
            website_url: valid_website,
            // stac_version,
            logo_url: valid_logo,
            description,
            catalog_ids,
            created_at: Some(get_datetime_utc()),
            updated_at: None,
        })
    }

    pub fn set_id(&mut self, id: String) {
        self.id = id
    }

    /// Apply a partial update. `None` fields are left alone.
    pub fn update(
        &mut self,
        name: Option<String>,
        website: Option<String>,
        logo_url: Option<String>,
        description: Option<String>,
        // stac_version: Option<String>,
        catalog_ids: Option<Vec<String>>,
    ) -> Result<(), ValidationError> {
        if let Some(updated_url) = logo_url {
            match parse_url(&updated_url) {
                Ok(valid_url) => {
                    self.logo_url = Some(valid_url.to_string());
                }
                Err(err) => {
                    return Err(ValidationError::InvalidUrl(err.to_string()));
                }
            }
        }

        if let Some(updated_url) = website {
            match parse_url(&updated_url) {
                Ok(valid_url) => {
                    self.website_url = Some(valid_url.to_string());
                }
                Err(err) => {
                    return Err(ValidationError::InvalidUrl(err.to_string()));
                }
            }
        }

        self.name = name;
        self.description = description;
        // self.stac_version = stac_version;
        self.catalog_ids = catalog_ids;
        self.set_update_date();
        Ok(())
    }

    /// Stamp `updated_at` with the current time.
    pub fn set_update_date(&mut self) {
        self.updated_at = Some(get_datetime_utc());
    }

    /// Stamp `created_at` with the current time.
    pub fn set_created_date(&mut self) {
        self.created_at = Some(get_datetime_utc());
    }

    /// Detach a catalog from this provider. No-op if not linked.
    pub fn remove_catalog(&mut self, catalog_id: &str) {
        if let Some(catalog_ids) = &mut self.catalog_ids {
            catalog_ids.retain(|id| id != catalog_id);

            if catalog_ids.is_empty() {
                self.catalog_ids = None;
            }
        }
    }

    /// Attach a catalog to this provider. No-op if already linked.
    pub fn add_catalog(&mut self, catalog_id: &str) {
        let catalog_ids = self.catalog_ids.get_or_insert_with(Vec::new);
       
        if !catalog_ids.iter().any(|id| id == &catalog_id) {
            catalog_ids.push(catalog_id.to_string());
             
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogProviderFilters {
    /// Performs an exact match on the `id` field.
    pub id: Option<String>,

    /// Performs a string search on the provider name.
    pub name: Option<String>,

    /// Performs a string search on the provider description.
    pub description: Option<String>,

    /// Performs a string search on the provider stac_version.
    // pub stac_version: Option<String>,

    /// Performs a string search on the provider catalogs.
    pub catalog_id: Option<String>,

    /// Performs a date search on the `created_at` field. Filters for for date `after` the provided date.
    pub created_after: Option<DateTime<Utc>>,
    /// Performs a date search on the `created_at` field. Filters for for date `before` the provided date.
    pub created_before: Option<DateTime<Utc>>,
    /// Performs a date search on the `updated_at` field. Filters for for date `after` the provided date.
    pub updated_after: Option<DateTime<Utc>>,
    /// Performs a date search on the `updated_at` field. Filters for for date `before` the provided date.
    pub updated_before: Option<DateTime<Utc>>,
}

impl Default for CatalogProviderFilters {
    fn default() -> Self {
        CatalogProviderFilters{
            id: None,
            name:None,
            // stac_version:None,
            catalog_id:None,
           
            description: None,
            
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogProviderUpdate {
    /// Updates the provider name.
    pub name: Option<String>,
    /// Updates the catalog description.
    pub description: Option<String>,
    /// Updates the provider logo url.
    pub logo_url: Option<String>,
    /// Updates the provider logo url.
    pub website_url: Option<String>,

    /// Updates the provider stac_version.
    // pub stac_version: Option<String>,

    /// Updates the provider stac_version.
    pub catalog_ids: Option<Vec<String>>,
}
