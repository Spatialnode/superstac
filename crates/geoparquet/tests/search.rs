use std::{collections::HashMap, fs::File};
use superstac_core::models::catalog::Catalog;
use superstac_geoparquet::GeoParquetBackend;
use superstac_search::{
    backend::{BackendSearchOptions, SearchBackend},
    query::SearchQuery,
};

fn query() -> SearchQuery {
    SearchQuery {
        collections: vec![],
        ids: None,
        intersects: None,
        bbox: None,
        datetime: None,
        limit: Some(10),
        sortby: None,
    }
}
fn options() -> BackendSearchOptions {
    BackendSearchOptions {
        max_items_per_catalog: 10,
        unify_response: true,
    }
}
fn item(id: &str, collection: &str, x: f64, datetime: &str) -> stac::Item {
    serde_json::from_value(serde_json::json!({
        "type": "Feature", "stac_version": "1.0.0", "id": id,
        "geometry": {"type": "Point", "coordinates": [x, 1.0]},
        "bbox": [x, 1.0, x, 1.0], "collection": collection,
        "properties": {"datetime": datetime}, "links": [],
        "assets": {"B02": {"href": "https://example.com/blue.tif"}}
    }))
    .unwrap()
}
fn setup(items: Vec<stac::Item>) -> (tempfile::TempDir, GeoParquetBackend, Catalog) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("items.parquet");
    stac::geoparquet::into_writer(File::create(&path).unwrap(), items).unwrap();
    let backend = GeoParquetBackend::new(HashMap::from([("test".into(), path)])).unwrap();
    let mut catalog = Catalog::new(
        "test",
        None::<String>,
        "https://example.com",
        None::<String>,
        None,
    )
    .unwrap();
    catalog
        .collection_aliases
        .insert("canonical".into(), "local".into());
    catalog.asset_aliases.insert(
        "canonical".into(),
        HashMap::from([("blue".into(), "B02".into())]),
    );
    (dir, backend, catalog)
}
fn items() -> Vec<stac::Item> {
    vec![
        item("outside", "local", 20.0, "2025-01-02T00:00:00Z"),
        item("old", "local", 1.0, "2020-01-02T00:00:00Z"),
        item("other", "other", 1.0, "2025-01-02T00:00:00Z"),
        item("match", "local", 1.0, "2025-01-02T00:00:00Z"),
    ]
}

#[tokio::test]
async fn filters_before_limiting_and_preserves_provenance_and_aliases() {
    let (_dir, backend, catalog) = setup(items());
    let mut q = query();
    q.collections = vec!["canonical".into()];
    q.bbox = Some(stac::Bbox::new(0.0, 0.0, 2.0, 2.0));
    q.datetime = Some("2025-01-01/2025-01-31".into());
    let opts = BackendSearchOptions {
        max_items_per_catalog: 1,
        ..options()
    };
    let results = backend.search(&catalog, q.clone(), opts).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].item.id, "match");
    assert_eq!(results[0].catalog_id, "test");
    assert_eq!(results[0].seen_in, ["test"]);
    assert_eq!(results[0].item.collection.as_deref(), Some("canonical"));
    assert!(results[0].item.assets.contains_key("blue"));
    let results = backend
        .search(
            &catalog,
            q,
            BackendSearchOptions {
                unify_response: false,
                ..opts
            },
        )
        .await
        .unwrap();
    assert_eq!(results[0].item.collection.as_deref(), Some("local"));
    assert!(results[0].item.assets.contains_key("B02"));
}

#[tokio::test]
async fn ids_intersects_empty_matches_and_zero_limit() {
    let (_dir, backend, catalog) = setup(items());
    let mut q = query();
    q.ids = Some(vec!["match".into()]);
    q.intersects = Some(
        serde_json::from_value(serde_json::json!({
            "type": "Polygon", "coordinates": [[[0,0],[2,0],[2,2],[0,2],[0,0]]]
        }))
        .unwrap(),
    );
    assert_eq!(
        backend
            .search(&catalog, q.clone(), options())
            .await
            .unwrap()
            .len(),
        1
    );
    q.ids = Some(vec!["outside".into()]);
    assert!(backend
        .search(&catalog, q, options())
        .await
        .unwrap()
        .is_empty());
    let mut q = query();
    q.limit = Some(0);
    assert!(backend
        .search(&catalog, q, options())
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn scans_more_than_one_batch() {
    let items = (0..1030)
        .map(|n| item(&format!("item-{n}"), "local", 1.0, "2025-01-02T00:00:00Z"))
        .collect();
    let (_dir, backend, catalog) = setup(items);
    let mut q = query();
    q.ids = Some(vec!["item-1029".into()]);
    q.limit = Some(1);
    let result = backend.search(&catalog, q, options()).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].item.id, "item-1029");
}

#[tokio::test]
async fn rejects_invalid_queries_and_corrupt_files() {
    let (dir, backend, catalog) = setup(items());
    let mut q = query();
    q.datetime = Some("invalid".into());
    assert!(backend.search(&catalog, q, options()).await.is_err());
    let mut q = query();
    q.sortby = Some(vec!["datetime".into()]);
    assert!(backend
        .search(&catalog, q, options())
        .await
        .unwrap_err()
        .to_string()
        .contains("sortby"));
    std::fs::write(dir.path().join("items.parquet"), b"corrupt").unwrap();
    assert!(backend.search(&catalog, query(), options()).await.is_err());
    std::fs::remove_file(dir.path().join("items.parquet")).unwrap();
    assert!(backend.search(&catalog, query(), options()).await.is_err());
}

#[test]
fn rejects_missing_paths_and_empty_configuration() {
    assert!(GeoParquetBackend::new(HashMap::new()).is_err());
    let dir = tempfile::tempdir().unwrap();
    assert!(
        GeoParquetBackend::new(HashMap::from([("test".into(), dir.path().join("missing"))]))
            .is_err()
    );
}
