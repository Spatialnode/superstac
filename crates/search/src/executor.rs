use futures::{StreamExt, TryStreamExt};
use stac::Item;
use superstac_core::{errors::SuperSTACError, models::catalog::Catalog};
use tokio::time::{sleep, timeout};

use crate::{
    aggregator::SearchAggregator,
    options::FederationOptions,
    query::SearchQuery,
    response::{CatalogFailure, SearchItem, SearchResponse},
    translator::to_stac_search,
    unifier,
};

/// Fans out a search across catalogs with retry, capped concurrency, and
/// per-catalog timeouts. Owns the `reqwest::Client` it borrows from the
/// engine so connection pools and default headers are shared.
pub struct SearchExecutor {
    client: reqwest::Client,
}

impl SearchExecutor {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// Query all `catalogs` concurrently and aggregate the results.
    /// Per-catalog failures are recorded on the response metadata's
    /// `failures` field rather than failing the whole call.
    pub async fn federated_search(
        &self,
        catalogs: Vec<Catalog>,
        query: SearchQuery,
        options: FederationOptions,
    ) -> Result<SearchResponse, SuperSTACError> {
        let catalogs_queried = catalogs.len();
        let concurrency = options.max_concurrent.max(1);

        tracing::debug!(
            catalogs = catalogs_queried,
            concurrency,
            collections = ?query.collections,
            "federated search"
        );

        let attempts = catalogs.into_iter().map(|catalog| {
            let query = query.clone();
            async move {
                let catalog_id = catalog.id.clone();
                let result = self
                    .search_catalog_with_retry(catalog, query, options)
                    .await;
                (catalog_id, result)
            }
        });

        let results: Vec<(String, Result<Vec<SearchItem>, SuperSTACError>)> =
            futures::stream::iter(attempts)
                .buffer_unordered(concurrency)
                .collect()
                .await;

        let mut successful = Vec::new();
        let mut failures: Vec<CatalogFailure> = Vec::new();

        for (catalog_id, outcome) in results {
            match outcome {
                Ok(items) => successful.push(items),
                Err(e) => failures.push(CatalogFailure {
                    catalog_id,
                    reason: e.to_string(),
                }),
            }
        }

        Ok(SearchAggregator::aggregate(
            successful,
            catalogs_queried,
            failures,
            options.deduplicate,
        ))
    }

    async fn search_catalog_with_retry(
        &self,
        catalog: Catalog,
        query: SearchQuery,
        options: FederationOptions,
    ) -> Result<Vec<SearchItem>, SuperSTACError> {
        let mut backoff = options.retry.initial_backoff;
        let mut last_error: SuperSTACError =
            SuperSTACError::SearchFailed("no attempts made".to_string());

        for attempt in 1..=options.retry.max_attempts {
            let attempt_result = timeout(
                options.per_catalog_timeout,
                self.search_catalog(catalog.clone(), query.clone(), options),
            )
            .await;

            match attempt_result {
                Ok(Ok(items)) => return Ok(items),
                Ok(Err(e)) => {
                    if !is_retryable(&e) || attempt == options.retry.max_attempts {
                        return Err(e);
                    }
                    last_error = e;
                }
                Err(_) => {
                    last_error = SuperSTACError::SearchFailed(format!(
                        "timeout after {:?}",
                        options.per_catalog_timeout
                    ));
                    if attempt == options.retry.max_attempts {
                        return Err(last_error);
                    }
                }
            }

            tracing::warn!(
                catalog = %catalog.id,
                attempt,
                error = %last_error,
                backoff_ms = backoff.as_millis() as u64,
                "search attempt failed, retrying"
            );

            sleep(backoff).await;
            backoff = (backoff * 2).min(options.retry.max_backoff);
        }

        Err(last_error)
    }

    async fn search_catalog(
        &self,
        catalog: Catalog,
        mut query: SearchQuery,
        options: FederationOptions,
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

        let items: Vec<SearchItem> = raw_items
            .into_iter()
            .map(|map_item| {
                let mut item: Item =
                    serde_json::from_value(serde_json::Value::Object(map_item))
                        .map_err(|err| SuperSTACError::SearchFailed(err.to_string()))?;

                if options.unify_response {
                    unifier::unify_item(&mut item, &catalog);
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

/// For now: treat all `SearchFailed` errors as retryable. The proper fix is a
/// richer error taxonomy (Network / Timeout / Server5xx / Client4xx) which is
/// out of scope here.
fn is_retryable(error: &SuperSTACError) -> bool {
    matches!(error, SuperSTACError::SearchFailed(_))
}
