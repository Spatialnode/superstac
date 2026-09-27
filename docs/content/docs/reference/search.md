---
title: Search parameters
description: Supported spatial, temporal, collection, and item filters and their exact limits.
---

Python clients take keyword arguments. Rust uses `SearchQuery`. Both are translated into a STAC API search for each selected catalog.

| Field | Python shape | Behavior |
| --- | --- | --- |
| `collections` | List of strings | Required by the Python query deserializer. Use canonical names, or `[]` for no collection filter. |
| `ids` | List of strings, optional | Filter by item IDs. |
| `bbox` | Coordinate list, optional | Bounding box. Use `[west, south, east, north]` in longitude/latitude for ordinary searches. |
| `intersects` | GeoJSON geometry dictionary, optional | Spatial geometry, not a Feature or FeatureCollection. |
| `datetime` | String, optional | Datetime instant or interval passed to upstream catalogs. |
| `limit` | Nonnegative integer, optional | Maximum collected items per catalog, default `10`; also capped by `max_items_per_catalog`. Prefer a positive value. |
| `sortby` | List of strings, optional | Present in the query model but **not forwarded by the current translator**. Do not rely on it. |

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

SuperSTAC follows upstream pagination internally until it collects `min(limit, max_items_per_catalog)` items from each catalog. There is no exposed federated cursor or lazy Python pagination API.

With three selected catalogs and `limit=10`, the engine may collect up to 30 items before deduplication. The final total is neither a complete match count nor guaranteed to equal your limit.

## Unsupported options

There is no implemented CQL2 filter, cloud-cover query, fields projection, global sort, or user-supplied search-header integration in the current engine path. Unknown Python dictionary keys may be ignored rather than rejected; only use the documented fields.

`SearchOptions` and `SearchRequest` exist in Rust source but are not wired into the engine's `search()` method. Do not treat their `headers` or `max_items` fields as active engine features.
