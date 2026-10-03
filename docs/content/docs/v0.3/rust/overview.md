---
title: Rust engine
description: Search STAC catalogs from a Rust application running on Tokio.
---

Use `SuperSTACEngine` to search catalogs from a Tokio application. It checks catalog health, discovers collections, and combines search results.

## Dependencies

Create an application first:

```bash
cargo new imagery-search
cd imagery-search
```

Add these dependencies to the generated `Cargo.toml`:

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
    let query = SearchQuery {
        collections: vec!["sentinel-2-l2a".into()],
        bbox: Some(vec![6.0, 45.0, 7.0, 46.0].try_into()?),
        datetime: Some(
            "2024-06-01T00:00:00Z/2024-06-30T23:59:59Z".into(),
        ),
        limit: Some(5),
        ids: None,
        intersects: None,
        sortby: None,
    };
    let result = engine.search(query).await;
    engine.shutdown().await;
    let result = result?;
    for entry in result.items {
        println!("{} from {:?}", entry.item.id, entry.seen_in);
    }
    for failure in result.metadata.failures {
        eprintln!("{}: {}", failure.catalog_id, failure.reason);
    }
    Ok(())
}
```

Run `cargo run` from the project root. The example prints item IDs, source catalogs, and any catalog failures.


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
| `superstac-geoparquet` | Save catalog metadata as GeoParquet, search local inventories, and compact or clean up datasets. |
| `superstac-cli` | Terminal interface over the engine. |

For local inventories or automatic selection between saved data and live catalogs, enable the engine's `geoparquet` feature. Replace the `superstac-engine` dependency above with:

```toml
superstac-engine = { version = "0.3", features = ["geoparquet"] }
```

The engine re-exports the ingestion and maintenance helpers, so most applications do not need a separate `superstac-geoparquet` dependency. Add that crate directly when working with its backend or dataset APIs. Follow the [complete Rust inventory example](/superstac/docs/guides/geoparquet#rust) to save and search a dataset.

Generate API documentation for the exact checkout with:

```bash
cargo doc --no-deps --features superstac-engine/geoparquet \
  -p superstac-core -p superstac-config -p superstac-search \
  -p superstac-engine -p superstac-geoparquet --open
```

The catalog registry currently uses memory storage. The `Sqlite` and `Postgres` variants also construct memory backends; they do not persist the registry. GeoParquet inventories are stored on disk separately.
