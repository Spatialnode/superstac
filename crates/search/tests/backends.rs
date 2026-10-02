use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use futures::future::BoxFuture;
use superstac_core::{errors::SuperSTACError, models::catalog::Catalog};
use superstac_search::{
    backend::{BackendSearchOptions, SearchBackend},
    executor::SearchExecutor,
    options::{FederationOptions, RetryPolicy},
    query::SearchQuery,
    response::SearchItem,
};

fn catalog(id: &str, url: &str) -> Catalog {
    Catalog::new(id, None::<String>, url, None::<String>, None).unwrap()
}

fn query() -> SearchQuery {
    SearchQuery {
        collections: vec!["canonical".into()],
        ids: None,
        intersects: None,
        bbox: None,
        datetime: None,
        limit: Some(5),
        sortby: None,
    }
}

struct TestBackend(Arc<AtomicUsize>);

impl SearchBackend for TestBackend {
    fn search<'a>(
        &'a self,
        catalog: &'a Catalog,
        query: SearchQuery,
        options: BackendSearchOptions,
    ) -> BoxFuture<'a, Result<Vec<SearchItem>, SuperSTACError>> {
        Box::pin(async move {
            assert_eq!(query.collections, ["canonical"]);
            assert_eq!(options.max_items_per_catalog, 3);
            assert!(options.unify_response);
            if catalog.id == "slow" {
                return futures::future::pending().await;
            }
            if self.0.fetch_add(1, Ordering::SeqCst) == 0 {
                return Err(SuperSTACError::SearchFailed("temporary failure".into()));
            }
            Ok(vec![SearchItem {
                catalog_id: catalog.id.clone(),
                seen_in: vec![catalog.id.clone()],
                item: stac::Item::new("scene"),
            }])
        })
    }
}

#[tokio::test]
async fn custom_backend_keeps_retries_timeouts_and_partial_results() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let executor = SearchExecutor::with_backend(Box::new(TestBackend(attempts.clone())));
    let options = FederationOptions {
        max_items_per_catalog: 3,
        per_catalog_timeout: Duration::from_millis(10),
        retry: RetryPolicy {
            max_attempts: 2,
            initial_backoff: Duration::ZERO,
            max_backoff: Duration::ZERO,
        },
        ..FederationOptions::default()
    };
    let response = executor
        .federated_search(
            vec![
                catalog("fast", "https://example.com"),
                catalog("slow", "https://example.com"),
            ],
            query(),
            options,
        )
        .await
        .unwrap();
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].catalog_id, "fast");
    assert_eq!(response.metadata.catalogs_queried, 2);
    assert_eq!(response.metadata.catalogs_succeeded, 1);
    assert_eq!(response.metadata.catalogs_failed, 1);
    assert_eq!(response.metadata.failures[0].catalog_id, "slow");
    assert!(response.metadata.failures[0].reason.contains("timeout"));
}

// A single-response local server keeps this regression independent of providers.
fn serve_items() -> (String, std::thread::JoinHandle<serde_json::Value>) {
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(std::time::Instant::now() < deadline, "no search request");
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => panic!("accept: {e}"),
            }
        };
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut bytes = Vec::new();
        let (body_start, length) = loop {
            let mut buf = [0; 4096];
            let n = socket.read(&mut buf).unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&buf[..n]);
            if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&bytes[..end]);
                assert!(headers.starts_with("POST /search "));
                let length: usize = headers
                    .lines()
                    .find_map(|line| {
                        let (key, value) = line.split_once(':')?;
                        key.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse().unwrap())
                    })
                    .unwrap();
                break (end + 4, length);
            }
        };
        while bytes.len() < body_start + length {
            let mut buf = [0; 4096];
            let n = socket.read(&mut buf).unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&buf[..n]);
        }
        let request = serde_json::from_slice(&bytes[body_start..body_start + length]).unwrap();
        let features: Vec<_> = ["one", "two"]
            .into_iter()
            .map(|id| {
                let mut item = stac::Item::new(id);
                item.collection = Some("local".into());
                item.assets.insert(
                    "B02".into(),
                    stac::Asset::new("https://example.com/blue.tif"),
                );
                item
            })
            .collect();
        let body =
            serde_json::json!({"type": "FeatureCollection", "features": features, "links": []})
                .to_string();
        write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
        request
    });
    (url, handle)
}

#[tokio::test]
async fn default_backend_preserves_aliases_caps_and_provenance() {
    for unify_response in [true, false] {
        let (url, server) = serve_items();
        let mut source = catalog("test", &url);
        source
            .collection_aliases
            .insert("canonical".into(), "local".into());
        source
            .asset_aliases
            .insert("canonical".into(), [("blue".into(), "B02".into())].into());
        let executor = SearchExecutor::new(reqwest::Client::builder().no_proxy().build().unwrap());
        let options = FederationOptions {
            max_items_per_catalog: 1,
            unify_response,
            retry: RetryPolicy {
                max_attempts: 1,
                ..RetryPolicy::default()
            },
            ..FederationOptions::default()
        };
        let response = executor
            .federated_search(vec![source], query(), options)
            .await
            .unwrap();
        let request = server.join().unwrap();
        assert_eq!(request["collections"], serde_json::json!(["local"]));
        assert_eq!(request["limit"], 1);
        assert_eq!(
            response.metadata.catalogs_failed, 0,
            "{:?}",
            response.metadata.failures
        );
        assert_eq!(response.items.len(), 1);
        let result = &response.items[0];
        assert_eq!(result.catalog_id, "test");
        assert_eq!(result.seen_in, ["test"]);
        assert_eq!(
            result.item.collection.as_deref(),
            Some(if unify_response { "canonical" } else { "local" })
        );
        assert!(result
            .item
            .assets
            .contains_key(if unify_response { "blue" } else { "B02" }));
    }
}
