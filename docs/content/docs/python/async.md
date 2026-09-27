---
title: Async Python
description: Use AsyncClient in asyncio applications while keeping local registry operations synchronous.
---

`AsyncClient` exposes the same search engine without blocking the asyncio event loop on network operations.

```python
import asyncio
from superstac import AsyncClient

async def main():
    client = AsyncClient(catalogs=[
        {"id": "earth-search", "url": "https://earth-search.aws.element84.com/v1"},
        {"id": "microsoft", "url": "https://planetarycomputer.microsoft.com/api/stac/v1"},
    ])
    try:
        await client.start()
        search = await client.search(collections=["sentinel-2-l2a"], limit=5)
        for item in search.items():
            print(item["id"])
        print(search.metadata)
    finally:
        await client.shutdown()

asyncio.run(main())
```

In a notebook that already has an event loop, use `await main()` instead of `asyncio.run(main())`.

## What needs await?

| Operation | AsyncClient behavior |
| --- | --- |
| `AsyncClient(...)` | Synchronous construction. |
| `AsyncClient.from_yaml(path)` | Synchronous construction and file loading. |
| `AsyncClient.open(url)` | Await; registers and starts a single-catalog client. |
| `start()`, `shutdown()`, `search(...)` | Await. |
| `list_collections()`, `get_collections()`, `get_collection(id)` | Await. |
| `catalogs_supporting(id)`, `collections_by_catalog()`, `describe_collection(catalog_id, collection_id)` | Await. |
| Catalog/provider CRUD and settings operations | Synchronous; access local memory. |
| `search.items()`, `search.to_geojson()`, `search.matched()` | Synchronous; results are already collected. |

## Open one endpoint

```python
import asyncio
from superstac import AsyncClient

async def main():
    client = await AsyncClient.open("https://earth-search.aws.element84.com/v1")
    try:
        print(await client.list_collections())
    finally:
        await client.shutdown()

asyncio.run(main())
```

Async search still collects each catalog's results before returning. It is not an async iterator over a federated result stream.
