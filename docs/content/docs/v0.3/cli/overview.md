---
title: Command-line interface
description: Search catalogs and inspect collections from your terminal.
---

Run `superstac` from your terminal. [Install it](/docs/v0.3/start/installation/) and save the [complete sample](/superstac/versions/v0.3/examples/superstac.yml) as `superstac.yml`.

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

For negative coordinates, use an equals sign: `--bbox=-10,35,5,45`. To sort, add `--sortby=-datetime` for newest first or `--sortby=eo:cloud_cover` for lowest cloud cover first. You can repeat `--sortby`; ordering applies within each catalog. The CLI does not accept `intersects`.

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

The JSON response contains `items` and `metadata`. Each entry in `items` contains the STAC record under `item`, plus `catalog_id` and `seen_in` to show where it came from. See [results](/docs/v0.3/reference/results) for an example.

Logs and ingestion progress go to stderr, leaving stdout available for JSON. Keep the two streams separate when piping into `jq` or another parser. `--quiet` reduces logging but still allows warnings.

`RUST_LOG` overrides the CLI-selected logging level when logging is enabled. A successful exit code can still come with failed catalogs. Check `metadata.failures` to see whether any catalog searches failed.

See [GeoParquet inventories](/docs/v0.3/guides/geoparquet) to save metadata, search it locally, and update saved inventories.
