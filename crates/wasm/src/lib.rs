//! JavaScript bindings for browser-hosted live STAC searches.
#![cfg(target_arch = "wasm32")]

use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use superstac_core::models::catalog::Catalog;
use superstac_search::{executor::SearchExecutor, options::FederationOptions, query::SearchQuery};
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    catalogs: Vec<CatalogConfig>,
    #[serde(default)]
    settings: Settings,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogConfig {
    id: String,
    url: String,
    #[serde(default)]
    collection_aliases: HashMap<String, String>,
    #[serde(default)]
    asset_aliases: HashMap<String, HashMap<String, String>>,
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Settings {
    deduplicate_items: bool,
    unify_response: bool,
    max_concurrent_catalogs: usize,
    per_catalog_timeout_seconds: u32,
    max_retry_attempts: u8,
    max_items_per_catalog: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            deduplicate_items: true,
            unify_response: true,
            max_concurrent_catalogs: 8,
            per_catalog_timeout_seconds: 30,
            max_retry_attempts: 2,
            max_items_per_catalog: 1000,
        }
    }
}

fn error(value: impl std::fmt::Display) -> JsError {
    JsError::new(&value.to_string())
}

fn to_js(value: &impl Serialize) -> Result<JsValue, JsError> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(error)
}

#[wasm_bindgen]
pub struct SuperSTAC {
    catalogs: Vec<Catalog>,
    client: reqwest::Client,
    options: FederationOptions,
}

#[wasm_bindgen]
impl SuperSTAC {
    #[wasm_bindgen(constructor)]
    pub fn new(config: ConfigInput) -> Result<SuperSTAC, JsError> {
        let config: Config = serde_wasm_bindgen::from_value(config.into()).map_err(error)?;
        let settings = config.settings;
        if config.catalogs.is_empty() {
            return Err(error("at least one catalog is required"));
        }
        if settings.max_concurrent_catalogs == 0
            || settings.per_catalog_timeout_seconds == 0
            || settings.max_retry_attempts == 0
            || settings.max_items_per_catalog == 0
        {
            return Err(error(
                "concurrency, timeout, attempts, and item cap must be positive",
            ));
        }
        let mut ids = HashSet::new();
        let mut catalogs = Vec::new();
        for input in config.catalogs {
            if !ids.insert(input.id.clone()) {
                return Err(error("duplicate catalog id"));
            }
            let url = reqwest::Url::parse(&input.url).map_err(error)?;
            if !matches!(url.scheme(), "http" | "https")
                || url.query().is_some()
                || url.fragment().is_some()
                || !url.username().is_empty()
                || url.password().is_some()
            {
                return Err(error("catalog URL must be an HTTP(S) base URL without credentials, query, or fragment"));
            }
            let mut catalog =
                Catalog::new(&input.id, None::<String>, &input.url, None::<String>, None)
                    .map_err(error)?;
            catalog.collection_aliases = input.collection_aliases;
            catalog.asset_aliases = input.asset_aliases;
            catalogs.push(catalog);
        }
        let mut options = FederationOptions::default();
        options.deduplicate = settings.deduplicate_items;
        options.unify_response = settings.unify_response;
        options.max_concurrent = settings.max_concurrent_catalogs;
        options.per_catalog_timeout =
            Duration::from_secs(settings.per_catalog_timeout_seconds.into());
        options.retry.max_attempts = settings.max_retry_attempts;
        options.max_items_per_catalog = settings.max_items_per_catalog;
        Ok(Self {
            catalogs,
            client: reqwest::Client::new(),
            options,
        })
    }

    /// Search all registered catalogs. Individual catalog failures are in metadata.
    pub async fn search(&self, query: QueryInput) -> Result<SearchOutput, JsError> {
        let mut value: serde_json::Value =
            serde_wasm_bindgen::from_value(query.into()).map_err(error)?;
        let object = value
            .as_object_mut()
            .ok_or_else(|| error("query must be an object"))?;
        for key in object.keys() {
            if !matches!(
                key.as_str(),
                "collections" | "ids" | "intersects" | "bbox" | "datetime" | "limit" | "sortby"
            ) {
                return Err(error(format!("unknown query field: {key}")));
            }
        }
        object
            .entry("collections")
            .or_insert_with(|| serde_json::json!([]));
        let query: SearchQuery = serde_json::from_value(value).map_err(error)?;
        if query.bbox.is_some() && query.intersects.is_some() {
            return Err(error("bbox and intersects cannot be combined"));
        }
        if query.limit == Some(0) {
            return Err(error("limit must be positive"));
        }
        let result = SearchExecutor::new(self.client.clone())
            .federated_search(self.catalogs.clone(), query, self.options)
            .await
            .map_err(error)?;
        Ok(to_js(&result)?.unchecked_into())
    }

    /// Discover collections per catalog, preserving failures and original metadata.
    #[wasm_bindgen(js_name = listCollections)]
    pub async fn list_collections(&self) -> Result<CollectionsOutput, JsError> {
        use futures::StreamExt;
        let results: Vec<serde_json::Value> = futures::stream::iter(self.catalogs.iter().map(|catalog| async move {
            let result = superstac_search::runtime::timeout(self.options.per_catalog_timeout,
                superstac_search::browser::collections(&self.client, &catalog.url, 10_000)).await;
            match result {
                Ok(Ok(collections)) => {
                    let collections: Vec<_> = collections.into_iter().map(|collection| {
                        let id = collection.get("id").and_then(|id| id.as_str()).unwrap_or("");
                        serde_json::json!({ "canonical_id": catalog.canonical_collection(id), "collection": collection })
                    }).collect();
                    serde_json::json!({ "catalog_id": catalog.id, "collections": collections, "error": null })
                }
                Ok(Err(e)) => serde_json::json!({ "catalog_id": catalog.id, "collections": [], "error": e.to_string() }),
                Err(e) => serde_json::json!({ "catalog_id": catalog.id, "collections": [], "error": e }),
            }
        })).buffered(self.options.max_concurrent).collect().await;
        Ok(to_js(&results)?.unchecked_into())
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "SuperSTACConfig")]
    pub type ConfigInput;
    #[wasm_bindgen(typescript_type = "SearchQuery")]
    pub type QueryInput;
    #[wasm_bindgen(typescript_type = "SearchResponse")]
    pub type SearchOutput;
    #[wasm_bindgen(typescript_type = "CatalogCollections[]")]
    pub type CollectionsOutput;
}

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &'static str = include_str!("types.d.ts");
