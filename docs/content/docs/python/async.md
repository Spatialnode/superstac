---
title: Async Python
description: Search catalogs without blocking your asyncio application.
---

Use `AsyncClient` when your application already uses asyncio. Await searches and other network operations so the event loop can keep running while catalogs respond.

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
| Adding, reading, updating, or deleting catalogs/providers; settings methods | Synchronous; access local memory. |
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

Once `await client.search(...)` returns, the results are in memory. Read them with a regular `for` loop; items do not arrive one at a time through an async iterator.
