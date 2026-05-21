use std::collections::HashMap;

use superstac_core::models::catalog::Catalog;

/// One collection ID aggregated across all catalogs that serve it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionAvailability {
    pub id: String,
    /// Catalog IDs that serve this collection (sorted).
    pub catalogs: Vec<String>,
}

/// Aggregate the introspected `supported_collections` across all catalogs into
/// a flat list of "collection -> serving catalogs." Catalogs whose
/// `supported_collections` is `None` (not yet introspected) contribute nothing.
pub fn aggregate_collections(catalogs: &[Catalog]) -> Vec<CollectionAvailability> {
    let mut by_id: HashMap<String, Vec<String>> = HashMap::new();
    for catalog in catalogs {
        if let Some(supported) = &catalog.supported_collections {
            for collection_id in supported {
                by_id
                    .entry(collection_id.clone())
                    .or_default()
                    .push(catalog.id.clone());
            }
        }
    }
    let mut result: Vec<CollectionAvailability> = by_id
        .into_iter()
        .map(|(id, mut catalogs)| {
            catalogs.sort();
            CollectionAvailability { id, catalogs }
        })
        .collect();
    result.sort_by(|a, b| a.id.cmp(&b.id));
    result
}

/// Return the IDs of every catalog whose introspected set contains
/// `collection_id`. Catalogs with `None` (uninitialized) are excluded — we
/// only report definite matches.
pub fn catalogs_supporting(catalogs: &[Catalog], collection_id: &str) -> Vec<String> {
    catalogs
        .iter()
        .filter(|c| {
            c.supported_collections
                .as_ref()
                .map(|s| s.contains(collection_id))
                .unwrap_or(false)
        })
        .map(|c| c.id.clone())
        .collect()
}

/// Per-catalog view: catalog ID -> sorted list of canonical collection IDs.
/// Catalogs with no introspection data are omitted.
pub fn collections_by_catalog(catalogs: &[Catalog]) -> HashMap<String, Vec<String>> {
    catalogs
        .iter()
        .filter_map(|c| {
            c.supported_collections.as_ref().map(|s| {
                let mut ids: Vec<String> = s.iter().cloned().collect();
                ids.sort();
                (c.id.clone(), ids)
            })
        })
        .collect()
}

/// Return the requested collections that no known-introspected catalog can
/// serve. Conservative: if any catalog has unknown (None) capabilities, we
/// don't flag the collection (it might be served).
pub fn unsupported_collections(catalogs: &[Catalog], requested: &[String]) -> Vec<String> {
    requested
        .iter()
        .filter(|q| {
            let could_be_supported = catalogs.iter().any(|c| match &c.supported_collections {
                None => true,
                Some(set) => set.contains(q.as_str()),
            });
            !could_be_supported
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn catalog(id: &str, supported: Option<&[&str]>) -> Catalog {
        let mut c = Catalog::new(id, Some(id), "https://example.com", None::<String>, None).unwrap();
        c.supported_collections = supported.map(|s| s.iter().map(|x| x.to_string()).collect());
        c
    }

    #[test]
    fn aggregate_collections_groups_by_id_and_sorts() {
        let catalogs = vec![
            catalog("a", Some(&["sentinel-2-l2a", "landsat-c2-l2"])),
            catalog("b", Some(&["sentinel-2-l2a", "naip"])),
            catalog("c", None), // contributes nothing
        ];

        let result = aggregate_collections(&catalogs);

        assert_eq!(result.len(), 3);
        // Output is sorted by collection id
        assert_eq!(result[0].id, "landsat-c2-l2");
        assert_eq!(result[0].catalogs, vec!["a"]);
        assert_eq!(result[1].id, "naip");
        assert_eq!(result[1].catalogs, vec!["b"]);
        assert_eq!(result[2].id, "sentinel-2-l2a");
        assert_eq!(result[2].catalogs, vec!["a", "b"]);
    }

    #[test]
    fn catalogs_supporting_returns_only_definite_matches() {
        let catalogs = vec![
            catalog("a", Some(&["sentinel-2-l2a"])),
            catalog("b", Some(&["landsat-c2-l2"])),
            catalog("c", None),
        ];

        assert_eq!(
            catalogs_supporting(&catalogs, "sentinel-2-l2a"),
            vec!["a".to_string()]
        );
        assert!(catalogs_supporting(&catalogs, "made-up").is_empty());
    }

    #[test]
    fn collections_by_catalog_skips_uninitialized() {
        let catalogs = vec![
            catalog("a", Some(&["x", "y"])),
            catalog("b", None),
        ];

        let map = collections_by_catalog(&catalogs);
        assert_eq!(map.len(), 1);
        assert_eq!(map["a"], vec!["x".to_string(), "y".to_string()]);
        assert!(!map.contains_key("b"));
    }

    #[test]
    fn unsupported_collections_is_conservative_when_uncertain() {
        let catalogs = vec![
            catalog("a", Some(&["sentinel-2-l2a"])),
            catalog("b", None), // uncertain — might support anything
        ];

        // typo collection — at least one catalog (b) is uncertain, so we
        // can't be sure it's unsupported. Don't flag.
        assert!(unsupported_collections(&catalogs, &["typo-collection".to_string()]).is_empty());
    }

    #[test]
    fn unsupported_collections_flags_when_all_catalogs_known() {
        let catalogs = vec![
            catalog("a", Some(&["sentinel-2-l2a"])),
            catalog("b", Some(&["landsat-c2-l2"])),
        ];

        let unsupported = unsupported_collections(
            &catalogs,
            &[
                "sentinel-2-l2a".to_string(),
                "made-up-collection".to_string(),
            ],
        );

        assert_eq!(unsupported, vec!["made-up-collection".to_string()]);
    }

    #[test]
    fn unsupported_collections_empty_for_empty_request() {
        let catalogs = vec![catalog("a", Some(&["x"]))];
        assert!(unsupported_collections(&catalogs, &[]).is_empty());
    }

    #[test]
    #[allow(dead_code)]
    fn aggregate_collections_no_panic_with_empty_input() {
        let _: HashSet<String> = HashSet::new(); // touch HashSet import
        assert!(aggregate_collections(&[]).is_empty());
    }
}
