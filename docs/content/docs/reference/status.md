---
title: Status and limitations
description: What the current alpha supports and what is still being built.
---

SuperSTAC is a pre-1.0, MIT-licensed project. This documentation targets version 0.3.0. APIs and YAML shapes may change.

## Available

- Browser WebAssembly binding for live STAC search, with JavaScript and TypeScript support.
- Python sync and async clients backed by Rust.
- GeoParquet inventories, coverage-aware local search, incremental overlays, compaction and cleanup.
- A Tokio-based Rust engine and command-line tool.
- In-memory catalog and provider registration.
- Collection discovery and source selection.
- Concurrent searches with per-catalog caps, retries, and timeouts.
- Collection/asset aliases and optional response normalization.
- Item-ID deduplication, failure metadata, and Rust/CLI item provenance.

The [browser binding](/docs/wasm/overview) supports live catalogs only. GeoParquet, filesystem configuration, background health monitoring, and asset signing are not included.

## Not implemented yet

- Persistent SQLite and Postgres backends. Rust enum variants exist but currently return memory storage.
- Integrated authentication, custom request headers, OAuth, or asset signing in the engine search path.
- Federated cursors, lazy Python streaming, and stable global sorting.
- CQL2, field projection, or cloud-cover search options.
- Full pystac-client API compatibility.

The package searches item metadata; it does not download, mosaic, render, or reproject imagery.

## Current alpha behavior to account for

YAML and Python dictionary configuration have different required fields. Python `matched()` counts returned records. Python item dictionaries omit per-item provenance wrappers. `sortby` orders results per catalog; the merged response is not globally sorted. Health monitors and global monitoring settings have limitations documented in [resilience](/docs/guides/resilience/).

These are implementation constraints, not promises about future releases. Track development in the [repository](https://github.com/spatialnode/superstac) and [changelog](https://github.com/spatialnode/superstac/blob/main/CHANGELOG.md).

## License

SuperSTAC uses the MIT license. See the [repository LICENSE](https://github.com/spatialnode/superstac/blob/main/LICENSE) for its terms.
