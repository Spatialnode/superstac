# SuperSTAC Roadmap

Our priority is reliable federated STAC search. These are areas of focus,
subject to feedback and capacity, with no fixed delivery dates.

Last updated: October 2, 2026

## Implemented for v0.3

- [x] [Reusable satellite-data inventories with GeoParquet](https://github.com/Spatialnode/superstac/issues/31): scoped ingestion with progress and resume, local search and API fallback, Python integration, scope unions, sorting, incremental refresh, compaction and cleanup.



## Short term

- [ ] [Support private catalogs](https://github.com/Spatialnode/superstac/issues/32) through per-catalog headers and environment-based credentials, consistently across search, discovery, health checks and ingestion.
- [ ] [Add AI/agent integration](https://github.com/Spatialnode/superstac/issues/22): define a supported tool interface for catalog discovery and bounded searches, with a working agent example. Existing Python APIs can be called by agents; a dedicated integration is still pending.
- [ ] [Explore WASM bindings](https://github.com/Spatialnode/superstac/issues/21): validate a browser-compatible API search client first, including CORS and runtime constraints. Native filesystem and GeoParquet support should not be assumed to work in browsers.

## Near term

- [ ] Add explicit preferred-catalog ordering and failover between providers. Improve recovery of catalogs unavailable at startup. Health checks, retries, concurrent search and snapshot-to-API fallback already exist.
- [ ] Implement SQLite persistence for catalog registration, provider configuration and settings. 
- [ ] Expose resumable federated pagination and bounded/lazy result iteration across Python, Rust and the CLI. Provider pagination is already followed internally, but callers currently receive materialized, capped results rather than a federated cursor or lazy stream.
- [ ] API

## Long term

Explore OAuth/token refresh and asset signing, additional configuration storage
backends, and ecosystem integrations as real use cases emerge. These extend the
basic credential support above rather than duplicate it.

## v1.0 release

- [ ] Stable public APIs and configuration format, with a compatibility policy.
- [ ] Release confidence in core search, with predictable results and failure handling. Existing tests are a foundation; scale, interoperability and failure-recovery coverage remain ongoing work.
- [ ] Maintain documentation, supported-platform coverage and migration guidance for every stable release. Documentation and packaging already exist; this is a continuing release requirement.

The v1.0 release depends on core stability; longer-term ideas are optional.
