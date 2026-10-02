# SuperSTAC (Python)

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/spatialnode/superstac/main/docs/assets/superstac-logo-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/spatialnode/superstac/main/docs/assets/superstac-logo.svg">
    <img src="https://raw.githubusercontent.com/spatialnode/superstac/main/docs/assets/superstac-logo.svg" alt="SuperSTAC logo" width="480">
  </picture>
</p>

**Many catalogs. One search.**

Search across STAC catalogs through one interface, with Python, Rust, or the command line.

This package provides the Python bindings for [SuperSTAC](https://github.com/spatialnode/superstac).

> **Status: alpha.** APIs are not yet stable. Pre-1.0; expect breaking changes.

`superstac` provides a synchronous `Client` and an `AsyncClient` for asyncio.
Results are Python dictionaries. Some method names resemble pystac-client, but
this is not a drop-in replacement.

## Documentation

Read the [SuperSTAC documentation](https://spatialnode.com/superstac) for installation, tutorials, and API guides.

- [Quickstart](https://spatialnode.com/superstac/docs/start/quickstart)
- [Python guide](https://spatialnode.com/superstac/docs/python/overview)
- [Rust guide](https://spatialnode.com/superstac/docs/rust/overview)
- [Command-line guide](https://spatialnode.com/superstac/docs/cli/overview)
- [Configuration reference](https://spatialnode.com/superstac/docs/reference/configuration)

Try the [Python quickstart notebook](https://spatialnode.com/superstac/docs/python/notebook) for a two-catalog search, footprint map, and GeoJSON export in Colab or Jupyter.

## Upgrading from 0.1.0a2

Version 0.2.0 replaces the old Python implementation with a Rust engine.
Use `from superstac import Client, AsyncClient` and follow the
[Python guide](https://spatialnode.com/superstac/docs/python/overview) to migrate.
Results are dictionaries, `matched()` counts returned items, and `to_geojson()`
exports a FeatureCollection. Custom authentication hooks and the full
pystac-client API are not supported. CPython 3.9 or newer is required.

## Install

```bash
pip install superstac
```

Or from source:

```bash
git clone https://github.com/spatialnode/superstac
cd superstac/crates/python
pip install maturin
maturin develop
```

## Quickstart

### Search a single catalog

```python
from superstac import Client

client = Client.open("https://earth-search.aws.element84.com/v1")

search = client.search(
    collections=["sentinel-2-l2a"],
    bbox=[6.0, 49.0, 7.0, 50.0],
    datetime="2024-01-01/2024-01-31",
    limit=50,
)

for item in search.items():
    print(item["id"], item["properties"]["datetime"])

print(search.matched(), "items")
fc = search.to_geojson()    # GeoJSON FeatureCollection dict
```

### Federated across multiple catalogs

Pass a YAML-shaped dict in one call:

```python
from superstac import Client

client = Client({
    "catalogs": [
        {"id": "earth-search", "url": "https://earth-search.aws.element84.com/v1"},
        {"id": "microsoft",    "url": "https://planetarycomputer.microsoft.com/api/stac/v1"},
    ],
    "settings": {"deduplicate_items": True, "max_concurrent_catalogs": 8},
})
client.start()

search = client.search(collections=["sentinel-2-l2a"], limit=50)
items = list(search.items())
print(search.metadata)   # per-catalog stats: queried, succeeded, failed, dedupes
```

Or build it up incrementally (any combination is fine):

```python
client = Client()
client.add_catalogs([
    {"id": "earth-search", "url": "https://earth-search.aws.element84.com/v1"},
    {"id": "microsoft",    "url": "https://planetarycomputer.microsoft.com/api/stac/v1"},
])
client.update_settings({"deduplicate_items": True})
client.start()
```

### Async

```python
import asyncio
from superstac import AsyncClient

async def main():
    client = await AsyncClient.open("https://earth-search.aws.element84.com/v1")
    search = await client.search(collections=["sentinel-2-l2a"], limit=50)
    for item in search.items():
        print(item["id"])
    await client.shutdown()

asyncio.run(main())
```

### Load from YAML

```python
client = Client.from_yaml("superstac.yml")          # sync
client = AsyncClient.from_yaml("superstac.yml")     # async
```

## License
MIT.

## GeoParquet inventories (v0.3)

Wheels include GeoParquet support (`superstac.geoparquet_available`). Both clients
accept `mode="snapshot"` or `mode="auto"`, `dataset="./data"`, and
`max_snapshot_age_seconds=86400`, including through `from_yaml()` and `open()`.
The default is `mode="live"` without a dataset.

```python
from superstac import Client

client = Client.from_yaml("superstac.yml")
client.ingest("earth-search", "./data", name="madrid",
              collections=["sentinel-2-l2a"],
              bbox=[-3.8, 40.3, -3.6, 40.5],
              datetime="2025-02-01/2025-02-08",
              progress=lambda event: print(event["phase"], event["items_saved"]))
local = Client.from_yaml("superstac.yml", mode="snapshot", dataset="./data")
result = local.search(collections=["sentinel-2-l2a"],
                      bbox=[-3.8, 40.3, -3.6, 40.5],
                      datetime="2025-02-01/2025-02-08", sortby=["-datetime"])
```

`AsyncClient.ingest`, `compact_dataset`, and `cleanup_dataset` are awaitable.
Use `incremental_since="2025-02-05T00:00:00Z"` with the same named scope to overlay
an acquisition-time window. This does not detect deletions or changes outside the
window; periodic full refreshes remain necessary. Auto mode falls back to live
APIs when complete fresh local coverage is unavailable, and does not persist live
results. Previews and imagery remain remote.

Compaction publishes new files; cleanup defaults to a preview:
`client.cleanup_dataset("./data")`. Pass `apply=True` to remove retired files
once snapshot clients have been released. `shutdown()` stops monitoring but does
not release the snapshot reader lock while the client remains alive.

See the [GeoParquet guide](https://spatialnode.com/superstac/docs/guides/geoparquet)
for scope coverage, progress, storage budgets, and maintenance semantics.
