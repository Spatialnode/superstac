---
title: Catalogs and discovery
description: Register STAC APIs, group them with providers, and discover which collections they serve.
---

## Register a catalog

Give each endpoint a distinct ID and an explicit URL. Use simple ASCII identifiers such as `earth-search` or `my-catalog`.

```python
from superstac import Client

client = Client()
client.add_catalog({
    "id": "earth-search",
    "title": "Earth Search",
    "url": "https://earth-search.aws.element84.com/v1",
})
try:
    client.start()
    print(client.list_collections())
finally:
    client.shutdown()
```

Always provide the root `url` of a compatible STAC API. A link to a static catalog JSON file will not work.

## Group catalogs by provider

Use providers to group catalogs by organization. Provider records hold names and descriptions, not login credentials.

```python
from superstac import Client

client = Client()
client.add_provider({"id": "element84", "name": "Element 84"})
client.add_catalog(
    {"id": "earth-search", "url": "https://earth-search.aws.element84.com/v1"},
    provider="element84",
)
print(client.list_catalogs())
```

Add the provider first, then pass its ID as `provider` when adding a catalog. The provider’s `catalog_ids` field is currently ignored when loading configuration.

## Discover collection availability

```python
from superstac import Client

client = Client.open(
    "https://earth-search.aws.element84.com/v1", id="earth-search"
)
try:
    print(client.list_collections())
    print(client.collections_by_catalog())
    print(client.catalogs_supporting("sentinel-2-l2a"))
    collection = client.describe_collection("earth-search", "sentinel-2-l2a")
    if collection is not None:
        print(collection["id"])
finally:
    client.shutdown()
```

`list_collections()` returns `{"id": ..., "catalogs": [...]}` records showing where each collection is available. Use `get_collections()` when you need full collection documents; it can make a separate request for each collection.

## Add, update, or remove catalogs

`add_catalogs()`, `get_catalog()`, `list_catalogs()`, `update_catalog()`, and `delete_catalog()` operate on the in-memory registry. Register catalogs before starting the client so the initial health checks and discovery cover them.

If you add catalogs after startup, call `shutdown()` and then `start()` to run startup discovery again. Calling `start()` while already started is a no-op.

Include the existing title and description when updating a catalog if you want to keep them; leaving them out can clear them. Changes are held in memory and disappear when you discard the client. Update your configuration file separately to keep them for the next run.

## Change your catalog list

This complete example adds a catalog, updates its description, searches it, then removes it. Register changes before `start()` or restart discovery afterward.

```python
from superstac import Client

client = Client()
try:
    client.add_catalog({
        "id": "earth-search",
        "title": "Earth Search",
        "url": "https://earth-search.aws.element84.com/v1",
    })
    client.update_catalog("earth-search", {
        "title": "Earth Search",
        "description": "Imagery for my project",
    })
    result = client.search(collections=["sentinel-2-l2a"], limit=1)
    print(len(result), result.metadata["failures"])
    client.shutdown()
    client.delete_catalog("earth-search")
    print(client.list_catalogs())
finally:
    client.shutdown()
```

The browser binding accepts catalogs in its constructor. To change them, finish pending calls, free the old client, and create a new one. It does not expose the Python registry methods. In the [playground](/docs/v0.3/wasm/playground), **Catalogs → Browse STAC Index** helps you find public APIs, and **Add by URL** accepts your own endpoint.
