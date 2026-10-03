---
title: Status and limitations
description: What the current alpha supports and what is still being built.
---

SuperSTAC is a pre-1.0, MIT-licensed project. This documentation targets version 0.3.0. APIs and YAML shapes may change.

## Available

- Python sync and async clients backed by Rust.
- Save metadata to GeoParquet, search it locally, update it, and remove old files.
- A Tokio-based Rust engine and command-line tool.
- In-memory catalog and provider registration.
- Collection discovery and source selection.
- Concurrent searches with per-catalog caps, retries, and timeouts.
- Collection/asset aliases and optional response normalization.
- Item-ID deduplication, failure metadata, and Rust/CLI item provenance.

## Not implemented yet

- Persistent SQLite and Postgres backends. Rust enum variants exist but currently return memory storage.
- Integrated authentication, custom request headers, OAuth, or asset signing in the engine search path.
- Federated cursors, lazy Python streaming, and stable global sorting.
- CQL2, field projection, or cloud-cover search options.
- Full pystac-client API compatibility.

The package searches item metadata; it does not download, mosaic, render, or reproject imagery.

## Things to check when using v0.3

- Start YAML files from the [sample configuration](/superstac/examples/superstac.yml). They require fields that Python dictionaries can omit.
- Python’s `matched()` counts returned items, not all matches in a catalog.
- Python results do not identify which catalog supplied each item.
- `sortby` sorts each catalog’s results separately.
- Catalogs that fail their initial health check need a restart to be checked again. See [health monitoring](/docs/guides/resilience/).

Follow changes in the [repository](https://github.com/spatialnode/superstac) and [changelog](https://github.com/spatialnode/superstac/blob/main/CHANGELOG.md).

## License

SuperSTAC uses the MIT license. See the [repository LICENSE](https://github.com/spatialnode/superstac/blob/main/LICENSE) for its terms.
