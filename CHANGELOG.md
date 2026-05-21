# Changelog

All notable changes to this project are documented here. Format inspired by
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[SemVer](https://semver.org/). Pre-1.0, expect breaking changes in minor
releases.

## [Unreleased]

## [0.1.0] - 2026-05-21

Initial public release.

### Added
- Workspace layout split across `superstac-core`, `superstac-config`,
  `superstac-search`, `superstac-engine`, and `superstac-cli`.
- Federated STAC search with concurrent fan-out, retry with exponential
  backoff, per-catalog timeouts, and concurrency cap.
- Collection alias rewriting (canonical ↔ local) so users can query with one
  name across heterogeneous catalogs.
- Asset alias rewriting for canonical asset key normalization.
- Item deduplication across catalogs with `seen_in` provenance.
- Response unification (collection + asset name rewriting on incoming items),
  settings-gated.
- Source selection: catalogs whose introspected `/collections` doesn't include
  the request are skipped.
- Per-catalog failure reporting via `SearchMetadata.failures`.
- Discovery API: `list_collections`, `catalogs_supporting`,
  `collections_by_catalog`, `describe_collection`.
- Background health monitoring with configurable check frequency.
- `superstac` CLI with `search` and `collections` subcommands, `--json`
  output, and `--verbose`/`--quiet` log control.
- `tracing` instrumentation across the engine, with `RUST_LOG` env var
  override.
- YAML configuration with shared workspace settings + per-catalog overrides.
