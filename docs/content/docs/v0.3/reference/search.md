---
title: Search parameters
description: Choose collections, areas, dates, item limits, and sort order for a search.
---

Pass filters as keyword arguments in Python, an object in JavaScript, a `SearchQuery` in Rust, or flags in the CLI. In live mode, SuperSTAC sends them to each selected catalog. For saved metadata, see the [GeoParquet guide](/docs/v0.3/guides/geoparquet).

| Field | Python shape | Behavior |
| --- | --- | --- |
| `collections` | List of strings | Required in Python. Use your configured collection names, or `[]` to search all collections. |
| `ids` | List of strings, optional | Filter by item IDs. |
| `bbox` | Coordinate list, optional | Bounding box. Use `[west, south, east, north]` in longitude/latitude for ordinary searches. |
| `intersects` | GeoJSON geometry dictionary, optional | Spatial geometry, not a Feature or FeatureCollection. |
| `datetime` | String, optional | Datetime instant or interval passed to upstream catalogs. |
| `limit` | Nonnegative integer, optional | Maximum collected items per catalog, default `10`; also capped by `max_items_per_catalog`. Prefer a positive value. |
| `sortby` | List of strings, optional | Per-catalog ordering, e.g. `["-datetime"]`. Forwarded to live APIs and supported locally; no global federated ordering. |

## Spatial and temporal filtering

```python
from superstac import Client

client = Client.open("https://earth-search.aws.element84.com/v1")
try:
    search = client.search(
        collections=["sentinel-2-l2a"],
        bbox=[6.0, 49.0, 7.0, 50.0],
        datetime="2024-01-01T00:00:00Z/2024-01-31T23:59:59Z",
        limit=20,
    )
    print(search.matched())
finally:
    client.shutdown()
```

Choose either `bbox` or `intersects` for a query. Use a closed GeoJSON polygon ring when passing a polygon. Coordinate and filter support ultimately depends on the target catalog.

## Limit and pagination

SuperSTAC reads result pages from each catalog until it reaches the smaller of `limit` and `max_items_per_catalog`, or there are no more items. Python receives the collected results at once; there is no cursor for requesting the next combined page.

With three catalogs and `limit=10`, you can get up to 30 items before duplicates are removed. The returned count can be smaller, and more matching items may exist in the catalogs.

## Examples you can adapt

[Search recipes](/docs/v0.3/guides/recipes) includes complete Python and JavaScript examples for a GeoJSON area, per-catalog sorting, looking up returned IDs, inspecting assets, and filtering the records you already fetched.

JavaScript uses the same field names. It can omit `collections`; Python requires an explicit list. The CLI supports the listed filters except `intersects`.

## Unsupported options

Search does not yet support CQL2 or cloud-cover filters, selecting which fields to return, sorting the combined results, or custom request headers. Use only the fields listed above: unknown Python keys may be silently ignored.

`SearchOptions` and `SearchRequest` exist in Rust source but are not wired into the engine's `search()` method. Do not treat their `headers` or `max_items` fields as active engine features.
