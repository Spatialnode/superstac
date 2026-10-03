---
title: Python client
description: Search multiple STAC APIs from scripts and notebooks with the synchronous Client.
---

Use `Client` to search STAC catalogs from a script or notebook. Results and collection documents are ordinary Python dictionaries.

For an interactive walkthrough, [try the Python notebook in Colab or Jupyter](/docs/v0.3/python/notebook). Search two catalogs, browse the items in a table, map their footprints, and export GeoJSON.

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

`open()` connects to the catalog and runs startup checks immediately. With `Client(...)` or `Client.from_yaml(...)`, those checks happen when you first search or discover collections.

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

You can also pass a configuration dictionary containing `catalogs`, `providers`, and `settings` as the first argument. Choose either that dictionary or the separate keyword arguments: when you supply `config`, the separate arguments are ignored.

## Work with the results

| Operation | Returns |
| --- | --- |
| `search.items()` | A list of STAC item dictionaries. |
| `search.matched()` | The number of returned items, after duplicates are removed if enabled. |
| `len(search)` | Number of returned items. |
| `search.to_geojson()` | A GeoJSON FeatureCollection dictionary. |
| `search.item_collection_as_dict()` | The same FeatureCollection representation. |
| `search.metadata` | Search counts, failures, and unsupported collection IDs. |

Iterate with `for item in search.items()`. Search collects the results before returning, so this loop reads items already in memory.

## Moving from pystac-client

If you use pystac-client, you will recognize `Client.open()`, `search()`, and `items()`. A few differences matter when switching: items are dictionaries rather than `pystac.Item` objects, and `matched()` counts returned items rather than all matches in the catalog. SuperSTAC does not support pystac-client’s full set of options, modifiers, authentication hooks, or pagination methods.

Use `to_geojson()`, not `as_geojson()`. See the [API reference](/docs/v0.3/python/api/) for supported methods and [async guide](/docs/v0.3/python/async/) for asyncio applications.
