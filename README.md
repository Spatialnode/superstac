# SuperSTAC

[![License](https://img.shields.io/badge/license-MIT-green.svg)](https://github.com/spatialnode/superstac/blob/main/LICENSE)
[![crates.io](https://img.shields.io/crates/v/superstac-search.svg)](https://crates.io/crates/superstac-search)
[![Documentation](https://img.shields.io/badge/docs-SuperSTAC-blue)](https://spatialnode.com/superstac)
[![docs.rs](https://docs.rs/superstac-search/badge.svg)](https://docs.rs/superstac-search)

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/spatialnode/superstac/main/docs/assets/superstac-logo-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/spatialnode/superstac/main/docs/assets/superstac-logo.svg">
    <img src="https://raw.githubusercontent.com/spatialnode/superstac/main/docs/assets/superstac-logo.svg" alt="SuperSTAC logo" width="480">
  </picture>
</p>

**Many catalogs. One search.**

Search across [STAC](https://stacspec.org/) catalogs through one interface, with Python, Rust, or the command line.

Query Element84, Microsoft Planetary Computer, and others through one API.
Items come back deduplicated, with their collection IDs and asset keys
normalized to canonical names — regardless of which catalog they came from.

> **Status: alpha.** APIs and YAML schema are not yet stable. Pre-1.0; expect
> breaking changes.

## Features

- **Federated search:** Query multiple STAC catalogs concurrently with spatial, temporal, collection, and item filters.
- **Collection discovery:** Find available collections and the catalogs that serve them.
- **Consistent names:** Map provider-specific collection IDs and asset keys to canonical names through configurable aliases.
- **Deduplicated results:** Merge items across catalogs by item ID, with source provenance available in Rust and CLI results.
- **Resilient requests:** Configure retries, per-catalog timeouts, and concurrency limits, with per-catalog failure reporting.
- **Python, Rust, and CLI:** Use synchronous or asynchronous Python clients, embed the Rust engine, or search from the command line.

## Why

A single STAC catalog isn't always enough:

- The collection you need lives somewhere else.
- The catalog you usually use is down or rate-limited.
- Different providers index the same scenes under different names.

SuperSTAC queries every catalog you've registered, drops the ones that don't
serve the requested collection, runs the rest concurrently with retry and
timeouts, then merges and dedupes the results.

## Documentation

Read the [SuperSTAC documentation](https://spatialnode.com/superstac) for installation, tutorials, and API guides.

- [Quickstart](https://spatialnode.com/superstac/docs/start/quickstart)
- [Python guide](https://spatialnode.com/superstac/docs/python/overview)
- [Rust guide](https://spatialnode.com/superstac/docs/rust/overview)
- [Command-line guide](https://spatialnode.com/superstac/docs/cli/overview)
- [Configuration reference](https://spatialnode.com/superstac/docs/reference/configuration)

Try the [Python quickstart notebook](https://spatialnode.com/superstac/docs/python/notebook) for a two-catalog search, footprint map, and GeoJSON export in Colab or Jupyter.

## Install

### Rust library

Add the crates you need:

```bash
cargo add superstac-core
cargo add superstac-search
cargo add superstac-engine
cargo add superstac-cli
cargo add superstac-config
```

### From source

```bash
git clone https://github.com/spatialnode/superstac
cd superstac
cargo build --release
```
## Quickstart

Drop a `superstac.yml` next to where you run the binary:

```yaml
catalogs:
  - id: earth-search
    url: https://earth-search.aws.element84.com/v1
  - id: microsoft
    url: https://planetarycomputer.microsoft.com/api/stac/v1
```

Then:

```bash
# what collections does each catalog serve?
superstac collections

# search across all of them
superstac search -c sentinel-2-l2a -b 6.0,49.0,7.0,50.0 -d 2024-01-01/2024-01-31 -l 50

# pipe to jq
superstac --json search -c landsat-c2-l2 -l 10 | jq '.metadata'

# inspect a single collection
superstac collections microsoft sentinel-2-l2a
```

Run `superstac --help` for the full surface.

## Configuration

Only `id` and `url` are required per catalog. Common optional fields:

```yaml
catalogs:
  - id: cdse
    url: catalog-url
    # Only needed when the catalog uses non-canonical names.
    collection_aliases:
      sentinel-2-l2a: S2MSI2A
    asset_aliases:
      sentinel-2-l2a:
        blue: B02
        green: B03
        red: B04

settings:
  health_check_strategy: "15m"
  deduplicate_items: true
  unify_response: true
  max_concurrent_catalogs: 8
  per_catalog_timeout_seconds: 30
  max_retry_attempts: 2
```

The full schema and every setting is documented inline at
[`crates/core/src/models/settings.rs`](https://github.com/spatialnode/superstac/blob/main/crates/core/src/models/settings.rs).

## Library usage

The CLI is a thin wrapper over [`superstac-engine`]. To embed in your own
binary:

```rust
use superstac_config::init_from_yaml;
use superstac_core::models::storage::Storage;
use superstac_engine::SuperSTACEngine;
use superstac_search::query::SearchQuery;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = init_from_yaml(Storage::Memory, "superstac.yml")?;
    let engine = SuperSTACEngine::new(db);
    engine.start().await?;

    let response = engine
        .search(SearchQuery {
            collections: vec!["sentinel-2-l2a".to_string()],
            limit: Some(20),
            bbox: None,
            datetime: None,
            ids: None,
            intersects: None,
            sortby: None,
        })
        .await?;

    println!("found {} items", response.metadata.total_items);
    Ok(())
}
```

## Crates

| Crate | Purpose |
|-------|---------|
| [`superstac-core`](https://crates.io/crates/superstac-core) | domain models, errors, storage trait |
| [`superstac-config`](https://crates.io/crates/superstac-config) | YAML config loading |
| [`superstac-search`](https://crates.io/crates/superstac-search) | federated search logic |
| [`superstac-engine`](https://crates.io/crates/superstac-engine) | runtime (health, introspection, search orchestration) |
| [`superstac-cli`](https://crates.io/crates/superstac-cli) | the `superstac` binary |

## Logs and debugging

Logs flow through `tracing`. The default level comes from `settings.log_level`
in your config; override at runtime:

```bash
superstac -v search -c sentinel-2-l2a       # debug
superstac -q search -c sentinel-2-l2a       # warn only
RUST_LOG=superstac_search=debug superstac search -c sentinel-2-l2a
```

## Roadmap

See [ROADMAP.MD](ROADMAP.MD) for short-term, near-term, and long-term priorities,
plus the v1.0 release checklist.

## License

MIT. See [LICENSE](https://github.com/spatialnode/superstac/blob/main/LICENSE).

Feedback and issues welcome — this is early. If you try it and you see any bug, feel free to open an issue!
