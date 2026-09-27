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

Catalog URLs must identify compatible STAC APIs, not arbitrary static catalog JSON files. Always supply `url`: although the current configuration type marks it optional, conversion assumes that it exists.

## Group catalogs by provider

Providers carry descriptive metadata; they are not authentication credentials.

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

Register providers before catalogs that refer to them. Prefer explicit catalog-to-provider assignment over the provider's `catalog_ids` input, which is not used during config conversion.

## Discover collection availability

```python
from superstac import Client

client = Client.open("https://earth-search.aws.element84.com/v1")
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

`list_collections()` returns a lightweight list of `{"id": ..., "catalogs": [...]}` records. `get_collections()` fetches full collection documents and can issue one HTTP request per distinct collection. Prefer the lightweight method when you only need availability.

## Change the registry

`add_catalogs()`, `get_catalog()`, `list_catalogs()`, `update_catalog()`, and `delete_catalog()` operate on the in-memory registry. Register catalogs before starting the client so the initial health checks and discovery cover them.

If you add catalogs after startup, call `shutdown()` and then `start()` to run startup discovery again. Calling `start()` while already started is a no-op.

Catalog updates are not fully patch-like yet: omitted title and description fields can be cleared. Preserve these fields explicitly when updating. Registry changes disappear when the client is discarded; persistent backends are not implemented.
