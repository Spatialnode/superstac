---
title: Command-line interface
description: Search catalogs and inspect collections from your terminal.
---

The binary is `superstac`. [Install it](/docs/start/installation/) and save the [complete sample](/superstac/examples/superstac.yml) as `superstac.yml`.

## Global options

| Option | Meaning |
| --- | --- |
| `--config PATH` | Configuration path; default `superstac.yml`. The basename must remain `superstac.yml` or `superstac.yaml`. |
| `--json` | Structured JSON output. |
| `-v`, `--verbose` | Debug logging. |
| `-q`, `--quiet` | Warning-level logging; conflicts with verbose. |
| `--help`, `--version` | Command help or binary version. |

## Search

```bash
superstac search -c sentinel-2-l2a -b 6.0,49.0,7.0,50.0 -l 10
superstac search -c sentinel-2-l2a -c landsat-c2-l2 -l 5
superstac search -c sentinel-2-l2a --id scene-id
```

| Option | Meaning |
| --- | --- |
| `-c`, `--collection ID` | Canonical collection ID; repeatable. Omit for no collection filter. |
| `-b`, `--bbox W,S,E,N` | Four comma-separated coordinates. |
| `-d`, `--datetime VALUE` | Datetime instant or interval forwarded to the catalog. |
| `-l`, `--limit N` | Maximum items per catalog; default `10`. |
| `--id ID` | Item ID; repeatable. |

For negative coordinates, use `--bbox=-10,35,5,45` to avoid flag parsing ambiguity. `intersects` is not a CLI option. Sorting accepts repeated `--sortby=-datetime` or `--sortby=eo:cloud_cover` flags.

## Collection discovery

```bash
# All discovered collections and their catalogs
superstac collections

# Collection IDs advertised by one catalog
superstac collections earth-search

# Full collection document
superstac collections earth-search sentinel-2-l2a
```

## JSON and logs

```bash
superstac --json search -c sentinel-2-l2a -l 5
RUST_LOG=superstac_search=debug superstac search -c sentinel-2-l2a
```

The JSON search response contains `items` and `metadata`. Each item is wrapped with `catalog_id`, `item`, and `seen_in`; it is not directly a GeoJSON FeatureCollection.

> **Current CLI logging caveat**
>
> The current tracing subscriber uses its default writer, which can put logs on standard output. For reliable JSON piping, set `logging_enabled: false` in your configuration before using `jq` or another parser. `--quiet` changes the level but does not disable all logs.

`RUST_LOG` overrides the CLI-selected logging level when logging is enabled. Per-catalog failures may appear inside a successfully serialized response; check the metadata rather than relying only on the exit code.

See [GeoParquet inventories](/docs/guides/geoparquet) for ingestion, backend selection, and dataset maintenance.
