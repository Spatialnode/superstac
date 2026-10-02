#![cfg(feature = "geoparquet")]
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    time::Duration,
};
use superstac_core::{
    models::{catalog::Catalog, storage::Storage},
    storages::factory::StorageBackend,
};
use superstac_engine::SuperSTACEngine;
use superstac_search::query::SearchQuery;

fn storage() -> Box<dyn StorageBackend> {
    let mut db = Storage::Memory.init();
    let mut catalog = Catalog::new(
        "snapshot",
        None::<String>,
        "http://127.0.0.1:9",
        None::<String>,
        None,
    )
    .unwrap();
    catalog.health_status.available = false;
    catalog.supported_collections = Some(HashSet::from(["wrong-remote-collection".into()]));
    db.create_catalog(catalog, None).unwrap();
    db.create_catalog(
        Catalog::new(
            "remote",
            None::<String>,
            "http://127.0.0.1:9",
            None::<String>,
            None,
        )
        .unwrap(),
        None,
    )
    .unwrap();

    db
}

#[tokio::test]
async fn snapshot_search_ignores_remote_health_and_inventory() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("items.parquet");
    let mut item = stac::Item::new("scene");
    item.collection = Some("actual".into());
    item.geometry = Some(
        serde_json::from_value(serde_json::json!({
            "type": "Point", "coordinates": [1.0, 1.0]
        }))
        .unwrap(),
    );
    stac::geoparquet::into_writer(File::create(&path).unwrap(), vec![item]).unwrap();
    let engine =
        SuperSTACEngine::from_geoparquet(storage(), HashMap::from([("snapshot".into(), path)]))
            .unwrap();
    let q = SearchQuery {
        collections: vec!["actual".into()],
        ids: None,
        intersects: None,
        bbox: None,
        datetime: None,
        limit: Some(10),
        sortby: None,
    };
    let response = tokio::time::timeout(Duration::from_secs(2), engine.search(q))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.metadata.catalogs_queried, 1);
    assert_eq!(
        response.metadata.catalogs_succeeded, 1,
        "{:?}",
        response.metadata.failures
    );
    assert!(response.metadata.unsupported_collections.is_empty());
    assert_eq!(response.items[0].item.id, "scene");
    assert_eq!(engine.list_collections().await.unwrap()[0].id, "actual");
    assert!(engine
        .describe_collection("snapshot", "actual")
        .await
        .unwrap().is_some());
    engine.shutdown().await;
}

#[test]
fn unknown_catalog_is_rejected() {
    assert!(SuperSTACEngine::from_geoparquet(
        storage(),
        HashMap::from([("unknown".into(), "missing.parquet".into())])
    )
    .is_err());
}
