use serde::Deserialize;

use superstac_core::models::{
    catalog::CatalogConfig,
    provider::CatalogProviderConfig,
    settings::Settings,
};

/// Configuration for SuperSTAC, Spatial Temporal Asset Catalog services
#[derive(Debug, Deserialize)]
pub struct SuperStacConfig {
    pub catalogs: Vec<CatalogConfig>,
    pub providers: Vec<CatalogProviderConfig>,
    pub settings: Settings,
}
