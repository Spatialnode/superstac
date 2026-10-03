use futures::StreamExt;
use superstac_core::{errors::SuperSTACError, models::catalog::Catalog};
use crate::runtime::{sleep, timeout};

use crate::{
    aggregator::SearchAggregator,
    backend::{BackendSearchOptions, SearchBackend},
    options::FederationOptions,
    query::SearchQuery,
    response::{CatalogFailure, SearchItem, SearchResponse},
    stac_api::StacApiBackend,
};

/// Fans out a search across catalogs with retry, capped concurrency, and
/// per-catalog timeouts. Delegates each retrieval attempt to a search backend.
pub struct SearchExecutor {
    backend: Box<dyn SearchBackend>,
}

impl SearchExecutor {
    pub fn new(client: reqwest::Client) -> Self {
        Self::with_backend(Box::new(StacApiBackend::new(client)))
    }

    /// Construct an executor with an alternative item retrieval backend.
    /// The backend handles every catalog passed to this executor; this does not
    /// configure engine source selection or health monitoring.
    pub fn with_backend(backend: Box<dyn SearchBackend>) -> Self {
        Self { backend }
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
                self.backend.search(
                    &catalog,
                    query.clone(),
                    BackendSearchOptions {
                        max_items_per_catalog: options.max_items_per_catalog,
                        unify_response: options.unify_response,
                    },
                ),
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
}

/// For now: treat all `SearchFailed` errors as retryable.
/// TODO - richer error taxonomy (Network / Timeout / Server5xx / Client4xx) and only retry on appropriate ones.
fn is_retryable(error: &SuperSTACError) -> bool {
    matches!(error, SuperSTACError::SearchFailed(_))
}
