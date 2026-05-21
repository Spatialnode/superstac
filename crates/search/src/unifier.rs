use stac::Item;
use superstac_core::models::catalog::Catalog;

/// Rewrite an item's `collection` and `assets` to canonical names using the
/// catalog's alias maps. No-op when the catalog declares no relevant aliases
/// or the item has no collection.
pub fn unify_item(item: &mut Item, catalog: &Catalog) {
    let canonical_collection = match &item.collection {
        Some(local) => {
            let canonical = catalog.canonical_collection(local).to_string();
            if &canonical != local {
                item.collection = Some(canonical.clone());
            }
            canonical
        }
        None => return,
    };

    let asset_map = match catalog.asset_aliases.get(&canonical_collection) {
        Some(map) if !map.is_empty() => map,
        _ => return,
    };

    // Build reverse: local_asset_key -> canonical_asset_key
    let reverse: std::collections::HashMap<&str, &str> = asset_map
        .iter()
        .map(|(canonical, local)| (local.as_str(), canonical.as_str()))
        .collect();

    // Drain and re-insert to rewrite keys in place (avoids depending on the
    // concrete map type used inside stac::Item).
    let pairs: Vec<_> = item.assets.drain(..).collect();
    for (key, asset) in pairs {
        let new_key = reverse
            .get(key.as_str())
            .map(|c| c.to_string())
            .unwrap_or(key);
        item.assets.insert(new_key, asset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stac::{Asset, Item};
    use std::collections::HashMap;

    fn make_catalog_with_aliases() -> Catalog {
        let mut catalog = Catalog::new(
            "cdse",
            Some("CDSE"),
            "https://cdse.example.com",
            None::<String>,
            None,
        )
        .unwrap();

        catalog
            .collection_aliases
            .insert("sentinel-2-l2a".to_string(), "S2MSI2A".to_string());

        let mut s2_assets = HashMap::new();
        s2_assets.insert("blue".to_string(), "B02".to_string());
        s2_assets.insert("green".to_string(), "B03".to_string());
        catalog
            .asset_aliases
            .insert("sentinel-2-l2a".to_string(), s2_assets);

        catalog
    }

    fn make_item_with_assets(collection: &str, keys: &[&str]) -> Item {
        let mut item = Item::new("scene-1");
        item.collection = Some(collection.to_string());
        for k in keys {
            item.assets.insert(
                k.to_string(),
                Asset::new(format!("https://example.com/{}.tif", k)),
            );
        }
        item
    }

    #[test]
    fn unify_rewrites_collection_to_canonical() {
        let catalog = make_catalog_with_aliases();
        let mut item = make_item_with_assets("S2MSI2A", &[]);

        unify_item(&mut item, &catalog);

        assert_eq!(item.collection.as_deref(), Some("sentinel-2-l2a"));
    }

    #[test]
    fn unify_rewrites_asset_keys_to_canonical() {
        let catalog = make_catalog_with_aliases();
        let mut item = make_item_with_assets("S2MSI2A", &["B02", "B03", "thumbnail"]);

        unify_item(&mut item, &catalog);

        assert!(item.assets.contains_key("blue"));
        assert!(item.assets.contains_key("green"));
        // Unmapped keys pass through unchanged.
        assert!(item.assets.contains_key("thumbnail"));
        assert!(!item.assets.contains_key("B02"));
    }

    #[test]
    fn unify_is_noop_when_collection_already_canonical_and_no_asset_rules() {
        let mut catalog = Catalog::new(
            "element84",
            Some("E84"),
            "https://example.com",
            None::<String>,
            None,
        )
        .unwrap();
        catalog
            .collection_aliases
            .insert("sentinel-2-l2a".to_string(), "sentinel-2-l2a".to_string());

        let mut item = make_item_with_assets("sentinel-2-l2a", &["blue", "green"]);
        let before_keys: Vec<_> = item.assets.keys().cloned().collect();

        unify_item(&mut item, &catalog);

        assert_eq!(item.collection.as_deref(), Some("sentinel-2-l2a"));
        let after_keys: Vec<_> = item.assets.keys().cloned().collect();
        assert_eq!(before_keys.len(), after_keys.len());
    }

    #[test]
    fn unify_is_noop_when_item_has_no_collection() {
        let catalog = make_catalog_with_aliases();
        let mut item = Item::new("scene-1");
        item.collection = None;

        unify_item(&mut item, &catalog);

        assert!(item.collection.is_none());
    }
}
