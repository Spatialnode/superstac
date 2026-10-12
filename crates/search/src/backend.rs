//! Search contracts shared by remote and file-backed implementations.
/// Futures run on the browser event loop on WASM; native backends remain Send.
#[cfg(not(target_arch = "wasm32"))]
pub type SearchFuture<'a, T> = futures::future::BoxFuture<'a, T>;
#[cfg(target_arch = "wasm32")]
pub type SearchFuture<'a, T> = futures::future::LocalBoxFuture<'a, T>;

#[cfg(not(target_arch = "wasm32"))]
pub trait BackendBounds: Send + Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send + Sync + ?Sized> BackendBounds for T {}
#[cfg(target_arch = "wasm32")]
pub trait BackendBounds {}
#[cfg(target_arch = "wasm32")]
impl<T: ?Sized> BackendBounds for T {}
use superstac_core::{errors::SuperSTACError, models::catalog::Catalog};

use crate::{query::SearchQuery, response::SearchItem};

/// Controls one retrieval attempt.
#[derive(Debug, Clone, Copy)]
pub struct BackendSearchOptions {
    pub max_items_per_catalog: usize,
    pub unify_response: bool,
}

/// Retrieves items from one catalog in a single attempt.
///
/// Queries use canonical collection names. Implementations resolve source-local
/// names, enforce the item cap, and preserve catalog provenance in results.
/// When requested, returned items use canonical collection and asset names.
/// The returned future must be safe to drop when the executor times out.
///
/// Dataset locations can be held by the backend; `Catalog::url` is only an API
/// endpoint for the STAC backend. This interface does not perform health checks
/// or ingestion. Boxed futures allow runtime selection via `dyn SearchBackend`.
pub trait SearchBackend: BackendBounds {
    fn search<'a>(
        &'a self,
        catalog: &'a Catalog,
        query: SearchQuery,
        options: BackendSearchOptions,
    ) -> SearchFuture<'a, Result<Vec<SearchItem>, SuperSTACError>>;
}

impl<T: SearchBackend + ?Sized> SearchBackend for std::sync::Arc<T> {
    fn search<'a>(
        &'a self,
        catalog: &'a Catalog,
        query: SearchQuery,
        options: BackendSearchOptions,
    ) -> SearchFuture<'a, Result<Vec<SearchItem>, SuperSTACError>> {
        (**self).search(catalog, query, options)
    }
}
