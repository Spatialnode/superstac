use std::collections::{HashMap, HashSet};

use reqwest::Client;
use stac::api::{Collections, UrlBuilder};
use superstac_core::{errors::SuperSTACError, models::catalog::Catalog};

/// Fetch the catalog's `/collections` endpoint and return the set of
/// **canonical** collection IDs it supports, after reversing any configured
/// alias map (catalog-local -> canonical).
/// TODO - What if collection is over 100, do I need to add a limit param and loop with pagination?
pub async fn fetch_supported_collections(
    client: &Client,
    catalog: &Catalog,
) -> Result<HashSet<String>, SuperSTACError> {
    let builder = UrlBuilder::new(&catalog.url)
        .map_err(|e| SuperSTACError::SearchFailed(format!("invalid catalog url: {}", e)))?;

    let response: Collections = client
        .get(builder.collections().as_str())
        .send()
        .await
        .map_err(|e| SuperSTACError::SearchFailed(format!("fetch /collections: {}", e)))?
        .error_for_status()
        .map_err(|e| SuperSTACError::SearchFailed(format!("/collections status: {}", e)))?
        .json()
        .await
        .map_err(|e| SuperSTACError::SearchFailed(format!("parse /collections: {}", e)))?;

    let reverse: HashMap<&str, &str> = catalog
        .collection_aliases
        .iter()
        .map(|(canonical, local)| (local.as_str(), canonical.as_str()))
        .collect();

    let canonical_ids: HashSet<String> = response
        .collections
        .into_iter()
        .map(|c| {
            reverse
                .get(c.id.as_str())
                .map(|canonical| canonical.to_string())
                .unwrap_or(c.id)
        })
        .collect();

    Ok(canonical_ids)
}
