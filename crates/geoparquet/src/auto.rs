//! Read-through selection without automatic persistence or partial-result merging.
use crate::GeoParquetBackend;
use futures::future::BoxFuture;
use std::{path::PathBuf, time::Duration};
use superstac_core::{errors::SuperSTACError, models::catalog::Catalog};
use superstac_search::{
    backend::{BackendSearchOptions, SearchBackend},
    query::SearchQuery,
    response::SearchItem,
    stac_api::StacApiBackend,
};

pub struct AutoBackend {
    root: PathBuf,
    max_age: Duration,
    live: StacApiBackend,
}
impl AutoBackend {
    pub fn new(root: PathBuf, max_age: Duration, client: reqwest::Client) -> Self {
        Self {
            root,
            max_age,
            live: StacApiBackend::new(client),
        }
    }
}
impl SearchBackend for AutoBackend {
    fn search<'a>(
        &'a self,
        catalog: &'a Catalog,
        query: SearchQuery,
        options: BackendSearchOptions,
    ) -> BoxFuture<'a, Result<Vec<SearchItem>, SuperSTACError>> {
        Box::pin(async move {
            crate::prepare_query(catalog, query.clone())?;
            if self.root.join("manifest.json").exists() {
                match GeoParquetBackend::from_dataset(&self.root) {
                    Ok(local) => {
                        if local.can_answer(catalog, query.clone(), self.max_age)? {
                            match local
                                .search_with_age(
                                    catalog,
                                    query.clone(),
                                    options,
                                    Some(self.max_age),
                                )
                                .await
                            {
                                Ok(items) => {
                                    tracing::info!(catalog = %catalog.id, source = "snapshot", "auto search source");
                                    return Ok(items);
                                }
                                Err(e) => {
                                    tracing::warn!(catalog = %catalog.id, error = %e, "snapshot unavailable; querying API")
                                }
                            }
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "dataset unavailable; querying API"),
                }
            }
            tracing::info!(catalog = %catalog.id, source = "live", "auto search source");
            self.live.search(catalog, query, options).await
        })
    }
}
