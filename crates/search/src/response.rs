use serde::{Deserialize, Serialize};
use stac::Item;

/// Outcome of a federated search: aggregated items plus run metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub items: Vec<SearchItem>,
    pub metadata: SearchMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchItem {
    /// The first-seen (primary) catalog. When this item was deduplicated,
    /// this is the catalog whose body we kept.
    pub catalog_id: String,
    pub item: Item,
    /// All catalogs that returned this item ID (always includes `catalog_id`).
    /// Length > 1 indicates this item was deduplicated across catalogs.
    pub seen_in: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMetadata {
    pub catalogs_queried: usize,
    pub catalogs_succeeded: usize,
    pub catalogs_failed: usize,
    /// Number of unique items in the response (post-dedup).
    pub total_items: usize,
    /// Number of items collapsed by deduplication.
    pub duplicates_removed: usize,
    /// Per-catalog failure details. Empty when all catalogs succeeded.
    pub failures: Vec<CatalogFailure>,
    /// Collection IDs from the query that no introspected catalog could serve.
    /// Populated by the engine post-aggregate. Conservative: only reported
    /// when every catalog had known capabilities and none matched.
    #[serde(default)]
    pub unsupported_collections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogFailure {
    pub catalog_id: String,
    pub reason: String,
}
