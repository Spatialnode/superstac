---
title: Rust engine
description: Embed SuperSTAC in an asynchronous Rust application with explicit configuration and lifecycle.
---

`SuperSTACEngine` orchestrates storage, health checks, collection discovery, and federated search. It runs on Tokio.

## Dependencies

```toml title="Cargo.toml"
[dependencies]
superstac-core = "0.2"
superstac-config = "0.2"
superstac-search = "0.2"
superstac-engine = "0.2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Use Rust 1.88 or newer as declared by the workspace. These examples target SuperSTAC 0.2.

## A complete search

Save the [quickstart configuration](/superstac/examples/superstac.yml) as `superstac.yml` next to where you run the binary.

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

`SuperSTACEngine::new` takes ownership of a boxed storage backend. `SuperSTACEngine::from_shared` accepts the exported `SharedStorage` handle for applications that need to retain registry access. Configure sources before startup; after changing the registry, restart the engine to refresh startup discovery.

## Crate boundaries

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
