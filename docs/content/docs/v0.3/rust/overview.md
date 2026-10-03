---
title: Rust engine
description: Search STAC catalogs from a Rust application running on Tokio.
---

Use `SuperSTACEngine` to search catalogs from a Tokio application. It checks catalog health, discovers collections, and combines search results.

## Dependencies

```toml title="Cargo.toml"
[dependencies]
superstac-core = "0.3"
superstac-config = "0.3"
superstac-search = "0.3"
superstac-engine = "0.3"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

You need Rust 1.88 or newer. These examples use SuperSTAC 0.3.

## A complete search

Save the [quickstart configuration](/superstac/versions/v0.3/examples/superstac.yml) as `superstac.yml` next to where you run the binary.

```rust title="src/main.rs"
use superstac_config::init_from_yaml;
use superstac_core::models::storage::Storage;
use superstac_engine::SuperSTACEngine;
use superstac_search::query::SearchQuery;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = init_from_yaml(Storage::Memory, "superstac.yml")?;
    let engine = SuperSTACEngine::new(storage);

    let result = engine.search(SearchQuery {
        collections: vec!["sentinel-2-l2a".into()],
        ids: None,
        intersects: None,
        bbox: None,
        datetime: Some("2024-01-01T00:00:00Z/2024-01-31T23:59:59Z".into()),
        limit: Some(10),
        sortby: None,
    }).await;

    engine.shutdown().await;
    let response = result?;
    for entry in response.items {
        println!("{} from {:?}", entry.item.id, entry.seen_in);
    }
    for failure in response.metadata.failures {
        eprintln!("{}: {}", failure.catalog_id, failure.reason);
    }
    Ok(())
}
```

Search starts the engine if needed. Explicit `engine.start().await?` is useful when you want to finish startup before serving application requests. `shutdown().await` cancels background health tasks.

## Discovery

| Method | Purpose |
| --- | --- |
| `list_collections().await` | Aggregated collection availability. |
| `collections_by_catalog().await` | Collections grouped by catalog. |
| `catalogs_supporting(id).await` | Catalogs advertising a canonical collection. |
| `describe_collection(catalog_id, collection_id).await` | Full collection document from one catalog. |

## Shared storage

Use `SuperSTACEngine::new` when the engine can own the boxed storage backend. Use `SuperSTACEngine::from_shared` with a `SharedStorage` handle when your application also needs access to the catalog registry. Add catalogs before startup; if you change them later, restart the engine to repeat discovery.

## Which crates to use

| Crate | Responsibility |
| --- | --- |
| `superstac-core` | Catalog/provider models, settings, errors, and storage traits. |
| `superstac-config` | YAML loading and backend population. |
| `superstac-search` | Query translation, concurrent execution, normalization, aggregation. |
| `superstac-engine` | Lifecycle, health, discovery, source selection. |
| `superstac-cli` | Terminal interface over the engine. |

Generate API documentation for the exact checkout with:

```bash
cargo doc --no-deps -p superstac-core -p superstac-config -p superstac-search -p superstac-engine --open
```

Only memory storage is implemented. The `Sqlite` and `Postgres` variants currently construct memory backends; they do not provide persistence.
