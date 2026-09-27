use std::collections::HashMap;

use serde::Deserialize;

use superstac_core::{errors::{SuperSTACError, ValidationError}, models::{
    catalog::{Catalog, CatalogSettings, get_default_health_status},
    provider::CatalogProvider,
    settings::Settings,
}, utils::{get_date_time, parse_url, validate_identifier}};


/// Configuration for SuperSTAC.
#[derive(Debug, Deserialize)]
pub struct SuperStacConfig {
    pub catalogs: Vec<CatalogConfig>,
    pub providers: Vec<CatalogProviderConfig>,
    pub settings: Settings,
}



/// A data structure for a STAC catalog from the YAML file.
#[derive(Debug, Deserialize)]
pub struct CatalogConfig {
    pub id: String,
    pub provider: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub settings: Option<CatalogSettings>,
    /// Maps canonical collection IDs to this catalog's local collection IDs.
    /// E.g. `sentinel-2-l2a: S2MSI2A` on a CDSE-style catalog.
    pub collection_aliases: Option<HashMap<String, String>>,
    /// Per-collection asset rename rules, keyed by canonical collection ID.
    /// Inner map: canonical asset key -> this catalog's local asset key.
    /// E.g. `{ "sentinel-2-l2a": { "blue": "B02", "green": "B03" } }`.
    pub asset_aliases: Option<HashMap<String, HashMap<String, String>>>,
}

impl TryFrom<CatalogConfig> for Catalog {
    type Error = SuperSTACError;

    fn try_from(cfg: CatalogConfig) -> Result<Self, Self::Error> {
        validate_identifier(&cfg.id)?;

        let url = match cfg.url {
            Some(w) => {
                parse_url(&w).map_err(|e| ValidationError::InvalidUrl(e.to_string()))?;
                Some(w)
            }
            None => None,
        };

        Ok(Self {
            id: cfg.id,
            provider: None,
            title: cfg.title,
            url: url.clone().unwrap(),
            description: cfg.description,
            settings: cfg.settings.unwrap_or(CatalogSettings::default()),
            health_status: get_default_health_status(url.unwrap()),
            capabilities: None,
            collection_aliases: cfg.collection_aliases.unwrap_or_default(),
            asset_aliases: cfg.asset_aliases.unwrap_or_default(),
            supported_collections: None,
            created_at: Some(get_date_time()),
            updated_at: None,
        })
    }
}


/// YAML-deserialization shape for a provider entry. Converted to
/// [`CatalogProvider`] via `TryFrom`.
#[derive(Debug, Deserialize)]
pub struct CatalogProviderConfig {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub website_url: Option<String>,
    // Disabling this for now, because it's probably better to have it in the catalog.
    // pub stac_version: Option<String>,
    pub catalog_ids: Option<Vec<String>>,
}

impl TryFrom<CatalogProviderConfig> for CatalogProvider {
    type Error = SuperSTACError;

    fn try_from(cfg: CatalogProviderConfig) -> Result<Self, Self::Error> {
        validate_identifier(&cfg.id)?;

        let website_url = match cfg.website_url {
            Some(w) => {
                parse_url(&w).map_err(|e| ValidationError::InvalidUrl(e.to_string()))?;
                Some(w)
            }
            None => None,
        };

        let logo_url = match cfg.logo_url {
            Some(l) => {
                parse_url(&l).map_err(|e| ValidationError::InvalidUrl(e.to_string()))?;
                Some(l)
            }
            None => None,
        };

        // let stac_version = cfg
        //     .stac_version
        //     .ok_or_else(|| ValidationError::MissingField("stac_version".into()))?;

        Ok(Self {
            id: cfg.id,
            name: cfg.name,
            description: cfg.description,
            website_url,
            logo_url,
            // stac_version: Some(stac_version),
            catalog_ids: None,
            created_at: Some(get_date_time()),
            updated_at: None,
        })
    }
}
