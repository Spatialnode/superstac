//! Item retrieval from a STAC API, including pagination.
#[cfg(not(target_arch = "wasm32"))]
use futures::{StreamExt, TryStreamExt};
use stac::Item;
use superstac_core::{errors::SuperSTACError, models::catalog::Catalog};

use crate::{
    backend::{BackendSearchOptions, SearchBackend, SearchFuture},
    query::SearchQuery,
    response::SearchItem,
    translator::to_stac_search,
    unifier,
};

/// Uses the engine's shared HTTP client and connection pool.
pub struct StacApiBackend {
    client: reqwest::Client,
}

impl StacApiBackend {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    async fn search_catalog(
        &self,
        catalog: &Catalog,
        mut query: SearchQuery,
        options: BackendSearchOptions,
    ) -> Result<Vec<SearchItem>, SuperSTACError> {
        // Translate canonical collection names to this catalog's local names.
        // Falls back to the canonical name when no alias is declared.
        query.collections = query
            .collections
            .iter()
            .map(|c| catalog.resolve_collection(c).to_string())
            .collect();

        // Cap items at min(user_limit, per-catalog system cap). Set on the
        // STAC search so the server doesn't waste a round-trip filling more
        // than we'd keep.
        let user_limit = query.limit.unwrap_or(10);
        let cap = user_limit.min(options.max_items_per_catalog);
        query.limit = Some(cap);

        let search = to_stac_search(query);

        if cap == 0 {
            return Ok(Vec::new());
        }
        #[cfg(not(target_arch = "wasm32"))]
        let raw_items: Vec<stac::api::Item> = {
            let stac_client = stac_io::api::Client::with_client(self.client.clone(), &catalog.url)
                .map_err(|e| SuperSTACError::SearchFailed(format!("stac client init: {}", e)))?;

            let stream = stac_client
                .search(search)
                .await
                .map_err(|e| SuperSTACError::SearchFailed(format!("search request: {}", e)))?;

            // Stream of `stac::api::Item` (= `serde_json::Map<String, Value>`),
            // paginated internally by stac-io. `take(cap)` bounds the total;
            // `try_collect` short-circuits on first per-item stream error.
            let raw_items: Vec<stac::api::Item> = stream
                .take(cap)
                .try_collect()
                .await
                .map_err(|e| SuperSTACError::SearchFailed(format!("stream item: {}", e)))?;

            raw_items
        };
        #[cfg(target_arch = "wasm32")]
        let raw_items = crate::browser::search(&self.client, &catalog.url, &search, cap).await?;

        let items: Vec<SearchItem> = raw_items
            .into_iter()
            .map(|map_item| {
                let mut item: Item = serde_json::from_value(serde_json::Value::Object(map_item))
                    .map_err(|err| SuperSTACError::SearchFailed(err.to_string()))?;

                if options.unify_response {
                    unifier::unify_item(&mut item, catalog);
                }

                Ok(SearchItem {
                    catalog_id: catalog.id.clone(),
                    seen_in: vec![catalog.id.clone()],
                    item,
                })
            })
            .collect::<Result<Vec<_>, SuperSTACError>>()?;

        Ok(items)
    }
}

impl SearchBackend for StacApiBackend {
    fn search<'a>(
        &'a self,
        catalog: &'a Catalog,
        query: SearchQuery,
        options: BackendSearchOptions,
    ) -> SearchFuture<'a, Result<Vec<SearchItem>, SuperSTACError>> {
        Box::pin(self.search_catalog(catalog, query, options))
    }
}
