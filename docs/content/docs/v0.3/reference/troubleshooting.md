---
title: Troubleshooting
description: Diagnose empty results, configuration errors, import failures, and unexpected search behavior.
---

## A search returns no items

Inspect `search.metadata` before changing the query:

- `catalogs_queried == 0`: no catalogs were searched. Check that you registered them, that they passed startup health checks, and that the collection IDs are correct.
- `catalogs_failed > 0`: read each entry in `failures` for the catalog and reason.
- `unsupported_collections` is nonempty: the catalogs checked did not advertise those collection names.
- Catalogs succeeded but returned no items: try a larger area or date range, and check that the catalogs have data for it.

Use `list_catalogs()` to inspect health and `list_collections()` for discovery. If a catalog’s `/collections` request failed, SuperSTAC may not know what it offers. That failure alone does not mean your collection is unavailable.

## YAML reports missing fields

Use the [complete sample](/superstac/versions/v0.3/examples/superstac.yml). YAML requires `catalogs`, `providers`, and `settings`, plus the six non-optional settings described in [configuration](/docs/v0.3/reference/configuration/). A minimal Python configuration dictionary is not a valid complete YAML file.

If you set a catalog's `settings`, include its frequency and status range. The file must be called `superstac.yml` or `superstac.yaml`.

## Python raises AttributeError for as_geojson

Use `search.to_geojson()` (or `search.item_collection_as_dict()`). Returned items are dictionaries, not PySTAC objects.

## Fewer items than expected

The limit is per catalog and bounded by `max_items_per_catalog`. Deduplication compares item IDs; it can collapse equal IDs even across different collections. Inspect `duplicates_removed` and try `deduplicate_items=False` if your sources reuse IDs.

## Sorting and cloud-cover filters

`sortby` is supported per catalog in v0.3; it does not globally sort the merged response. The engine does not implement a cloud-cover/CQL2 filter interface. Unsupported Python keyword arguments may be ignored. Use only [documented search fields](/docs/v0.3/reference/search/).

## A catalog never recovers

Catalogs unhealthy at startup currently do not get a background monitor. Shut down and restart the client to repeat health checks. Check per-catalog monitor settings; the global monitor update is not applied by memory storage in this alpha.

## JSON piping fails

Use `--json` for machine-readable output. CLI logs and ingestion progress go to stderr; avoid merging stderr into stdout when piping JSON. `--quiet` does not disable warnings.

## Python cannot load the extension

Use Python 3.9+ and ensure the package is installed in the interpreter you are running. Check `python -m pip show superstac`. If a compatible wheel is unavailable, follow the [source build instructions](/docs/v0.3/start/installation/).

## A browser search fails but the URL opens in a tab

A catalog can allow navigation while blocking requests from another website. Inspect the failed request in Developer Tools → Network. The API must allow your app’s origin through CORS, including POST searches. SuperSTAC cannot change a catalog’s CORS policy; use a catalog that supports browser access or a proxy you control.

## The WASM module does not load

Check the `.wasm` request in Developer Tools. Serve it over HTTP(S), ensure it returns the binary rather than an HTML page, and follow your bundler’s asset-URL rules. The [complete browser example](/docs/v0.3/wasm/overview#put-it-on-a-page) shows an explicit URL with Vite.

## Report a problem

Include the package version or commit, Python/Rust version, operating system, a minimal configuration without credentials, the query, and failure metadata. Open an issue on [GitHub](https://github.com/spatialnode/superstac/issues).

## Identify your version

```sh
superstac --version
python -c "import superstac; print(superstac.__version__)"
```

Rust applications can report `superstac_engine::VERSION`. Search response metadata
includes `superstac_version`, including searches with per-catalog failures.
Native provider requests use `User-Agent: superstac/<version>`. Browser requests keep the browser’s User-Agent. This identifies the
library, not an individual user, and does not send telemetry to SuperSTAC.

Newly ingested or compacted Parquet files record `created_by` and a JSON
`superstac` metadata entry with their writer version and write time. Existing
files are not changed until rewritten. Include that version when reporting a
file-reading problem; it may differ from your currently installed version.
