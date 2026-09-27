---
title: Python client
description: Search multiple STAC APIs from scripts and notebooks with the synchronous Client.
---

The Python package wraps the Rust engine using PyO3. Items and collection documents are returned as ordinary Python dictionaries.

For an interactive walkthrough, [try the Python notebook in Colab or Jupyter](/docs/python/notebook). It includes a federated search, item table, footprint map, and GeoJSON export.

## Connect to one catalog

```python
from superstac import Client

client = Client.open("https://earth-search.aws.element84.com/v1")
try:
    search = client.search(collections=["sentinel-2-l2a"], limit=10)
    for item in search.items():
        print(item["id"], list(item.get("assets", {})))
finally:
    client.shutdown()
```

`open()` registers the endpoint and starts the engine. `Client(...)` and `Client.from_yaml(...)` construct an unstarted client; the first search or discovery operation starts it automatically.

## Connect to multiple catalogs

```python
from superstac import Client

client = Client(
    catalogs=[
        {"id": "earth-search", "url": "https://earth-search.aws.element84.com/v1"},
        {"id": "microsoft", "url": "https://planetarycomputer.microsoft.com/api/stac/v1"},
    ],
    settings={"max_concurrent_catalogs": 4, "deduplicate_items": True},
)
try:
    search = client.search(collections=["sentinel-2-l2a"], limit=10)
    print(search.matched())
    print(search.metadata["duplicates_removed"])
finally:
    client.shutdown()
```

A positional configuration dictionary can contain `catalogs`, `providers`, and `settings`. If you pass `config`, it takes precedence over the separate keyword arguments. Choose one style per client.

## Work with the results

| Operation | Returns |
| --- | --- |
| `search.items()` | A materialized list of STAC item dictionaries. |
| `search.matched()` | The number of items returned after aggregation, not the upstream total. |
| `len(search)` | Number of returned items. |
| `search.to_geojson()` | A GeoJSON FeatureCollection dictionary. |
| `search.item_collection_as_dict()` | The same FeatureCollection representation. |
| `search.metadata` | Search counts, failures, and unsupported collection IDs. |

Iterate with `for item in search.items()`. The Python API does not stream items as they arrive; the Rust search has completed before the `Search` object is returned.

## Moving from pystac-client

The familiar `Client.open()`, `search()`, and `items()` shapes make migration easier, but this is **not full drop-in compatibility**. Returned items are dictionaries, not `pystac.Item` instances. `matched()` reports collected items, and arbitrary pystac-client options, modifiers, authentication hooks, and pagination APIs are not implemented.

Use `to_geojson()`, not `as_geojson()`. See the [API reference](/docs/python/api/) for the supported surface and [async guide](/docs/python/async/) for asyncio applications.
