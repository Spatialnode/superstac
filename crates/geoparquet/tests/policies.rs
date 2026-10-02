use chrono::Utc;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    time::Duration,
};
use superstac_core::models::catalog::Catalog;
use superstac_geoparquet::{
    auto::AutoBackend,
    ingest::{ingest_catalog, IngestOptions},
    manifest::{DataFile, DatasetManifest, IngestScope, Snapshot},
    progress::IngestPhase,
    GeoParquetBackend,
};
use superstac_search::{
    backend::{BackendSearchOptions, SearchBackend},
    query::SearchQuery,
};

fn item(id: &str, collection: &str) -> stac::Item {
    serde_json::from_value(json!({"type":"Feature", "stac_version":"1.0.0", "id":id,
        "collection":collection, "geometry":{"type":"Point","coordinates":[1,1]},
        "bbox":[1,1,1,1], "properties":{"datetime":"2025-01-02T00:00:00Z"},
        "links":[], "assets":{}}))
    .unwrap()
}
fn page(id: &str, collection: &str) -> Value {
    json!({"type":"FeatureCollection","features":[item(id,collection)],"links":[],"numberMatched":1})
}
fn server(responses: Vec<Value>) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let thread = std::thread::spawn(move || {
        for response in responses {
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            let mut socket = loop {
                match listener.accept() {
                    Ok((socket, _)) => break socket,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(std::time::Instant::now() < deadline, "missing API request");
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => panic!("{e}"),
                }
            };
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            loop {
                let mut buf = [0; 4096];
                let n = socket.read(&mut buf).unwrap();
                assert!(n > 0);
                request.extend_from_slice(&buf[..n]);
                if let Some(end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&request[..end]);
                    assert!(headers.starts_with("POST /search "));
                    assert!(headers.to_ascii_lowercase().contains(&format!(
                        "user-agent: superstac/{}",
                        env!("CARGO_PKG_VERSION")
                    )));
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            let (key, value) = line.split_once(':')?;
                            key.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse().unwrap())
                        })
                        .unwrap();
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            let body = response.to_string();
            write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
        }
    });
    (url, thread)
}
fn catalog(url: &str) -> Catalog {
    Catalog::new("test", None::<String>, url, None::<String>, None).unwrap()
}
fn query(collection: &str) -> SearchQuery {
    SearchQuery {
        collections: vec![collection.into()],
        ids: None,
        intersects: None,
        bbox: None,
        datetime: None,
        limit: Some(10),
        sortby: None,
    }
}
fn search_options() -> BackendSearchOptions {
    BackendSearchOptions {
        max_items_per_catalog: 10,
        unify_response: true,
    }
}
fn ingest_options(root: &std::path::Path, name: &str, collection: &str) -> IngestOptions {
    let mut options = IngestOptions::new(
        root,
        IngestScope {
            collections: vec![collection.into()],
            ..Default::default()
        },
    );
    options.name = Some(name.into());
    options
}

#[tokio::test]
async fn retains_scopes_refreshes_one_and_reports_durable_progress() {
    let (url, thread) = server(vec![page("a1", "a"), page("b1", "b"), page("a2", "a")]);
    let source = catalog(&url);
    let dir = tempfile::tempdir().unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut options = ingest_options(dir.path(), "scope-a", "a");
    let observed = events.clone();
    options.progress = Some(Arc::new(move |event| {
        observed.lock().unwrap().push(event.clone())
    }));
    ingest_catalog(&source, options).await.unwrap();
    ingest_catalog(&source, ingest_options(dir.path(), "scope-b", "b"))
        .await
        .unwrap();
    ingest_catalog(&source, ingest_options(dir.path(), "scope-a", "a"))
        .await
        .unwrap();
    thread.join().unwrap();
    let manifest = DatasetManifest::read(dir.path()).unwrap();
    assert_eq!(manifest.version, 2);
    assert_eq!(manifest.catalogs["test"].len(), 2);
    let path = dir
        .path()
        .join(&manifest.catalogs["test"]["scope-a"].files[0].path);
    let reader = parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(
        File::open(path).unwrap(),
    )
    .unwrap();
    let footer = reader.metadata().file_metadata();
    assert_eq!(
        footer.created_by(),
        Some(concat!("superstac/", env!("CARGO_PKG_VERSION")))
    );
    let kv = footer.key_value_metadata().unwrap();
    assert!(kv.iter().any(|v| v.key == "geo"));
    assert!(kv.iter().any(|v| v.key == "stac-geoparquet"));
    let producer: Value = serde_json::from_str(
        kv.iter()
            .find(|v| v.key == "superstac")
            .unwrap()
            .value
            .as_ref()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(producer["version"], env!("CARGO_PKG_VERSION"));
    let backend = GeoParquetBackend::from_dataset(dir.path()).unwrap();
    assert_eq!(
        backend
            .search(&source, query("a"), search_options())
            .await
            .unwrap()[0]
            .item
            .id,
        "a2"
    );
    assert_eq!(
        backend
            .search(&source, query("b"), search_options())
            .await
            .unwrap()[0]
            .item
            .id,
        "b1"
    );
    let mut union = query("a");
    union.collections.push("b".into());
    assert_eq!(
        backend
            .search(&source, union, search_options())
            .await
            .unwrap()
            .len(),
        2
    );
    let events = events.lock().unwrap();
    assert!(events
        .iter()
        .any(|e| e.items_received == 1 && e.items_saved == 0));
    assert_eq!(events.last().unwrap().phase, IngestPhase::Completed);
    assert_eq!(events.last().unwrap().items_saved, 1);
    assert_eq!(events.last().unwrap().total_items, Some(1));
}

fn local_snapshot(root: &std::path::Path, source: &Catalog) -> Snapshot {
    let path = root.join("items.parquet");
    stac::geoparquet::into_writer(File::create(&path).unwrap(), vec![item("local", "a")]).unwrap();
    Snapshot {
        name: "one".into(),
        catalog_id: source.id.clone(),
        source_url: source.url.clone(),
        scope: IngestScope {
            collections: vec!["a".into()],
            ..Default::default()
        },
        started_at: Utc::now(),
        completed_at: Some(Utc::now()),
        complete: true,
        overlays: false,
        pages: 1,
        items: 1,
        files: vec![DataFile {
            path: "items.parquet".into(),
            collection: Some("a".into()),
            items: 1,
            bytes: path.metadata().unwrap().len(),
            bbox: None,
            datetime_min: None,
            datetime_max: None,
        }],
    }
}
fn publish(root: &std::path::Path, snapshot: Snapshot) {
    let manifest = DatasetManifest {
        version: 2,
        catalogs: BTreeMap::from([(
            "test".into(),
            BTreeMap::from([(snapshot.name.clone(), snapshot)]),
        )]),
    };
    std::fs::write(
        root.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
}

#[tokio::test]
async fn auto_uses_fresh_scope_and_falls_back_for_stale_missing_or_broken_data() {
    let (url, thread) = server(vec![
        page("stale-live", "a"),
        page("miss-live", "b"),
        page("broken-live", "a"),
    ]);
    let source = catalog(&url);
    let dir = tempfile::tempdir().unwrap();
    publish(dir.path(), local_snapshot(dir.path(), &source));
    let backend = AutoBackend::new(
        dir.path().into(),
        Duration::from_secs(86400),
        reqwest::Client::builder().no_proxy().user_agent(concat!("superstac/", env!("CARGO_PKG_VERSION"))).build().unwrap(),
    );
    assert_eq!(
        backend
            .search(&source, query("a"), search_options())
            .await
            .unwrap()[0]
            .item
            .id,
        "local"
    );
    let stale = AutoBackend::new(
        dir.path().into(),
        Duration::ZERO,
        reqwest::Client::builder().no_proxy().user_agent(concat!("superstac/", env!("CARGO_PKG_VERSION"))).build().unwrap(),
    );
    assert_eq!(
        stale
            .search(&source, query("a"), search_options())
            .await
            .unwrap()[0]
            .item
            .id,
        "stale-live"
    );
    assert_eq!(
        backend
            .search(&source, query("b"), search_options())
            .await
            .unwrap()[0]
            .item
            .id,
        "miss-live"
    );
    let before = std::fs::read(dir.path().join("manifest.json")).unwrap();
    std::fs::remove_file(dir.path().join("items.parquet")).unwrap();
    assert_eq!(
        backend
            .search(&source, query("a"), search_options())
            .await
            .unwrap()[0]
            .item
            .id,
        "broken-live"
    );
    assert_eq!(
        before,
        std::fs::read(dir.path().join("manifest.json")).unwrap()
    );
    thread.join().unwrap();
}

#[tokio::test]
async fn reads_v1_manifests_as_legacy_scope() {
    let source = catalog("http://127.0.0.1:9");
    let dir = tempfile::tempdir().unwrap();
    let mut snapshot = serde_json::to_value(local_snapshot(dir.path(), &source)).unwrap();
    snapshot.as_object_mut().unwrap().remove("name");
    std::fs::write(
        dir.path().join("manifest.json"),
        json!({"version":1,"catalogs":{"test":snapshot}}).to_string(),
    )
    .unwrap();
    let manifest = DatasetManifest::read(dir.path()).unwrap();
    assert!(manifest.catalogs["test"].contains_key("legacy"));
    let backend = GeoParquetBackend::from_dataset(dir.path()).unwrap();
    assert_eq!(
        backend
            .search(&source, query("a"), search_options())
            .await
            .unwrap()[0]
            .item
            .id,
        "local"
    );
}

#[tokio::test]
async fn budget_failure_preserves_published_data_and_can_resume() {
    let (url, thread) = server(vec![page("old", "a"), page("new", "a"), page("new", "a")]);
    let source = catalog(&url);
    let dir = tempfile::tempdir().unwrap();
    let first = ingest_catalog(&source, ingest_options(dir.path(), "one", "a"))
        .await
        .unwrap();
    let before = std::fs::read(dir.path().join("manifest.json")).unwrap();
    let mut options = ingest_options(dir.path(), "one", "a");
    options.max_dataset_bytes = Some(first.files.iter().map(|f| f.bytes).sum::<u64>() + 1);
    assert!(ingest_catalog(&source, options)
        .await
        .unwrap_err()
        .to_string()
        .contains("budget"));
    assert_eq!(
        before,
        std::fs::read(dir.path().join("manifest.json")).unwrap()
    );
    let mut options = ingest_options(dir.path(), "one", "a");
    options.resume = true;
    ingest_catalog(&source, options).await.unwrap();
    thread.join().unwrap();
    let backend = GeoParquetBackend::from_dataset(dir.path()).unwrap();
    assert_eq!(
        backend
            .search(&source, query("a"), search_options())
            .await
            .unwrap()[0]
            .item
            .id,
        "new"
    );
}

#[tokio::test]
async fn incremental_overlay_compaction_cleanup_and_reader_protection() {
    let mut moved = item("same", "a");
    moved
        .set_geometry(Some(
            serde_json::from_value(json!({"type":"Point","coordinates":[20,20]})).unwrap(),
        ))
        .unwrap();
    let delta = json!({"type":"FeatureCollection","features":[moved,item("new","a")],"links":[]});
    let first = json!({"type":"FeatureCollection","features":[item("same","a"),item("old","a")],"links":[]});
    let (url, thread) = server(vec![first, delta]);
    let source = catalog(&url);
    let dir = tempfile::tempdir().unwrap();
    let original = ingest_catalog(&source, ingest_options(dir.path(), "one", "a"))
        .await
        .unwrap();
    let mut options = ingest_options(dir.path(), "one", "a");
    options.incremental_since = Some("2025-01-01T00:00:00Z".into());
    let refreshed = ingest_catalog(&source, options).await.unwrap();
    thread.join().unwrap();
    assert_eq!(original.started_at, refreshed.started_at);
    assert!(refreshed.overlays);
    let backend = GeoParquetBackend::from_dataset(dir.path()).unwrap();
    let mut q = query("a");
    q.bbox = Some(stac::Bbox::TwoDimensional([0., 0., 2., 2.]));
    let result = backend.search(&source, q, search_options()).await.unwrap();
    assert_eq!(result.len(), 2);
    assert!(
        !result.iter().any(|i| i.item.id == "same"),
        "old moved item must not reappear"
    );
    let mut q = query("a");
    q.sortby = Some(vec!["-id".into()]);
    q.limit = Some(2);
    let result = backend
        .search(&source, q.clone(), search_options())
        .await
        .unwrap();
    assert_eq!(
        result
            .iter()
            .map(|i| i.item.id.as_str())
            .collect::<Vec<_>>(),
        vec!["same", "old"]
    );
    let manifest =
        superstac_geoparquet::maintenance::compact(dir.path(), 100, Some(1048576)).unwrap();
    assert_eq!(manifest.catalogs["test"]["one"].items, 3);
    assert!(!manifest.catalogs["test"]["one"].overlays);
    assert!(superstac_geoparquet::maintenance::cleanup(dir.path(), true).is_err());
    // Old readers remain usable until dropped.
    assert_eq!(
        backend
            .search(&source, q.clone(), search_options())
            .await
            .unwrap()
            .len(),
        2
    );
    drop(backend);
    let dry = superstac_geoparquet::maintenance::cleanup(dir.path(), false).unwrap();
    assert!(dry.bytes > 0);
    assert!(!dry.applied);
    let applied = superstac_geoparquet::maintenance::cleanup(dir.path(), true).unwrap();
    assert_eq!(applied.files, dry.files);
    let backend = GeoParquetBackend::from_dataset(dir.path()).unwrap();
    assert_eq!(
        backend
            .search(&source, q, search_options())
            .await
            .unwrap()
            .len(),
        2
    );
    let collections = backend.collections(&source).unwrap();
    assert!(collections.contains_key("a"));
}

#[tokio::test]
async fn temporal_union_requires_gap_free_coverage() {
    let dir = tempfile::tempdir().unwrap();
    let source = catalog("http://127.0.0.1:9");
    let mut left = local_snapshot(dir.path(), &source);
    left.name = "left".into();
    left.scope.datetime = Some("2025-01-01T00:00:00Z/2025-01-15T00:00:00Z".into());
    let mut right = left.clone();
    right.name = "right".into();
    right.scope.datetime = Some("2025-01-15T00:00:00Z/2025-02-01T00:00:00Z".into());
    let mut manifest = DatasetManifest {
        version: 2,
        catalogs: BTreeMap::from([(
            "test".into(),
            BTreeMap::from([("left".into(), left), ("right".into(), right)]),
        )]),
    };
    std::fs::write(
        dir.path().join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let mut q = query("a");
    q.datetime = Some("2025-01-01T00:00:00Z/2025-02-01T00:00:00Z".into());
    let backend = GeoParquetBackend::from_dataset(dir.path()).unwrap();
    assert!(backend
        .can_answer(&source, q.clone(), Duration::from_secs(86400))
        .unwrap());
    assert_eq!(
        backend
            .search(&source, q.clone(), search_options())
            .await
            .unwrap()
            .len(),
        1
    );
    manifest
        .catalogs
        .get_mut("test")
        .unwrap()
        .get_mut("right")
        .unwrap()
        .scope
        .datetime = Some("2025-01-16T00:00:00Z/2025-02-01T00:00:00Z".into());
    std::fs::write(
        dir.path().join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let backend = GeoParquetBackend::from_dataset(dir.path()).unwrap();
    assert!(!backend
        .can_answer(&source, q, Duration::from_secs(86400))
        .unwrap());
}

#[tokio::test]
async fn union_rejects_spatial_holes_and_unknown_collections() {
    let dir = tempfile::tempdir().unwrap();
    let source = catalog("http://127.0.0.1:9");
    let mut a = local_snapshot(dir.path(), &source);
    a.name = "a".into();
    a.scope.bbox = Some(stac::Bbox::TwoDimensional([0., 0., 1., 1.]));
    let mut b = a.clone();
    b.name = "b".into();
    b.scope.bbox = Some(stac::Bbox::TwoDimensional([1., 1., 2., 2.]));
    let mut manifest = DatasetManifest {
        version: 2,
        catalogs: BTreeMap::from([(
            "test".into(),
            BTreeMap::from([("a".into(), a), ("b".into(), b)]),
        )]),
    };
    std::fs::write(
        dir.path().join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let mut q = query("a");
    q.bbox = Some(stac::Bbox::TwoDimensional([0., 0., 2., 2.]));
    let backend = GeoParquetBackend::from_dataset(dir.path()).unwrap();
    assert!(!backend
        .can_answer(&source, q.clone(), Duration::from_secs(60))
        .unwrap());
    manifest
        .catalogs
        .get_mut("test")
        .unwrap()
        .get_mut("a")
        .unwrap()
        .scope
        .bbox = Some(stac::Bbox::TwoDimensional([0., 0., 1., 2.]));
    manifest
        .catalogs
        .get_mut("test")
        .unwrap()
        .get_mut("b")
        .unwrap()
        .scope
        .bbox = Some(stac::Bbox::TwoDimensional([1., 0., 2., 2.]));
    std::fs::write(
        dir.path().join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let backend = GeoParquetBackend::from_dataset(dir.path()).unwrap();
    assert!(backend
        .can_answer(&source, q.clone(), Duration::from_secs(60))
        .unwrap());
    q.collections.clear();
    assert!(!backend
        .can_answer(&source, q, Duration::from_secs(60))
        .unwrap());
}

#[tokio::test]
async fn maintenance_protects_checkpoints_and_failed_compaction_preserves_manifest() {
    let (url, thread) = server(vec![page("old", "a")]);
    let source = catalog(&url);
    let dir = tempfile::tempdir().unwrap();
    let snapshot = ingest_catalog(&source, ingest_options(dir.path(), "one", "a"))
        .await
        .unwrap();
    thread.join().unwrap();
    let before = std::fs::read(dir.path().join("manifest.json")).unwrap();
    let bytes = snapshot.files.iter().map(|f| f.bytes).sum::<u64>();
    assert!(superstac_geoparquet::maintenance::compact(dir.path(), 10, Some(bytes + 1)).is_err());
    assert_eq!(
        before,
        std::fs::read(dir.path().join("manifest.json")).unwrap()
    );
    let run = dir.path().join("runs/checkpoint");
    std::fs::create_dir(&run).unwrap();
    let protected = run.join("protected.parquet");
    std::fs::copy(dir.path().join(&snapshot.files[0].path), &protected).unwrap();
    let mut partial = snapshot.clone();
    partial.files[0].path = "runs/checkpoint/protected.parquet".into();
    std::fs::write(
        dir.path().join("checkpoints/interrupted.json"),
        json!({"snapshot":partial}).to_string(),
    )
    .unwrap();
    let orphan = run.join("orphan.parquet");
    std::fs::copy(&protected, &orphan).unwrap();
    let report = superstac_geoparquet::maintenance::cleanup(dir.path(), true).unwrap();
    assert!(report
        .files
        .contains(&"runs/checkpoint/orphan.parquet".into()));
    assert!(protected.exists());
    assert!(!orphan.exists());
    assert_eq!(
        before,
        std::fs::read(dir.path().join("manifest.json")).unwrap()
    );
}
