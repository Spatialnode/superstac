use serde::Deserialize;
use stac::{Bbox, Geometry};

use crate::options::SearchOptions;

/// A federated STAC search request. Mirrors the STAC API search params:
/// the engine translates this per-catalog before dispatch.
///
/// `collections` should use **canonical** names — alias-mapping happens
/// internally per catalog.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchQuery {
    pub collections: Vec<String>,
    pub ids: Option<Vec<String>>,
    pub intersects: Option<Geometry>,
    pub bbox: Option<Bbox>,
    pub datetime: Option<String>,
    pub limit: Option<usize>,
    pub sortby: Option<Vec<String>>,
}


// TODO - Integrate it into the engine.
pub struct SearchRequest {
    pub query: SearchQuery,
    pub options: SearchOptions,
}