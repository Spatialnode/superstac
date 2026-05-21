use std::collections::HashMap;

use crate::response::{CatalogFailure, SearchItem, SearchMetadata, SearchResponse};

/// Folds per-catalog results into a single [`SearchResponse`]. Stateless;
/// kept as a struct so future tweaks (sorting, ranking) have a natural home.
pub struct SearchAggregator;

impl SearchAggregator {
    /// Flatten + dedup (if requested) and emit metadata.
    pub fn aggregate(
        results: Vec<Vec<SearchItem>>,
        catalogs_queried: usize,
        failures: Vec<CatalogFailure>,
        deduplicate: bool,
    ) -> SearchResponse {
        let flat: Vec<SearchItem> = results.into_iter().flatten().collect();
        let pre_dedup_count = flat.len();

        let items = if deduplicate {
            Self::dedup(flat)
        } else {
            flat
        };

        let total_items = items.len();
        let duplicates_removed = pre_dedup_count - total_items;
        let catalogs_failed = failures.len();

        SearchResponse {
            items,
            metadata: SearchMetadata {
                catalogs_queried,
                catalogs_succeeded: catalogs_queried - catalogs_failed,
                catalogs_failed,
                total_items,
                duplicates_removed,
                failures,
                // Filled in by the engine post-aggregate.
                unsupported_collections: Vec::new(),
            },
        }
    }

    /// Collapse items sharing the same `item.id`. First-seen wins; later
    /// occurrences contribute their `catalog_id` to the primary's `seen_in`.
    fn dedup(items: Vec<SearchItem>) -> Vec<SearchItem> {
        let mut id_to_idx: HashMap<String, usize> = HashMap::new();
        let mut out: Vec<SearchItem> = Vec::with_capacity(items.len());

        for item in items {
            let id = item.item.id.clone();
            if let Some(&idx) = id_to_idx.get(&id) {
                out[idx].seen_in.extend(item.seen_in);
            } else {
                id_to_idx.insert(id, out.len());
                out.push(item);
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stac::Item;

    fn make_item(catalog: &str, id: &str) -> SearchItem {
        SearchItem {
            catalog_id: catalog.to_string(),
            seen_in: vec![catalog.to_string()],
            item: Item::new(id),
        }
    }

    #[test]
    fn aggregate_with_dedup_collapses_shared_ids() {
        let results = vec![
            vec![make_item("a", "scene-1"), make_item("a", "scene-2")],
            vec![make_item("b", "scene-1"), make_item("b", "scene-3")],
        ];

        let resp = SearchAggregator::aggregate(results, 2, vec![], true);

        assert_eq!(resp.metadata.total_items, 3);
        assert_eq!(resp.metadata.duplicates_removed, 1);
        assert_eq!(resp.items.len(), 3);

        let scene1 = resp.items.iter().find(|i| i.item.id == "scene-1").unwrap();
        assert_eq!(scene1.catalog_id, "a");
        assert_eq!(scene1.seen_in, vec!["a", "b"]);

        let scene2 = resp.items.iter().find(|i| i.item.id == "scene-2").unwrap();
        assert_eq!(scene2.seen_in, vec!["a"]);
    }

    #[test]
    fn aggregate_without_dedup_keeps_duplicates() {
        let results = vec![
            vec![make_item("a", "scene-1")],
            vec![make_item("b", "scene-1")],
        ];

        let resp = SearchAggregator::aggregate(results, 2, vec![], false);

        assert_eq!(resp.metadata.total_items, 2);
        assert_eq!(resp.metadata.duplicates_removed, 0);
        assert_eq!(resp.items.len(), 2);
    }

    #[test]
    fn aggregate_records_per_catalog_failures() {
        let results = vec![vec![make_item("a", "scene-1")]];
        let failures = vec![
            CatalogFailure {
                catalog_id: "b".to_string(),
                reason: "timeout".to_string(),
            },
            CatalogFailure {
                catalog_id: "c".to_string(),
                reason: "503 Service Unavailable".to_string(),
            },
        ];

        let resp = SearchAggregator::aggregate(results, 3, failures, true);

        assert_eq!(resp.metadata.catalogs_queried, 3);
        assert_eq!(resp.metadata.catalogs_succeeded, 1);
        assert_eq!(resp.metadata.catalogs_failed, 2);
        assert_eq!(resp.metadata.failures.len(), 2);
        assert_eq!(resp.metadata.failures[0].catalog_id, "b");
        assert_eq!(resp.metadata.failures[1].reason, "503 Service Unavailable");
    }
}
