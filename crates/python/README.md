# SuperSTAC (Python)

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/spatialnode/superstac/main/docs/assets/superstac-logo-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/spatialnode/superstac/main/docs/assets/superstac-logo.svg">
    <img src="https://raw.githubusercontent.com/spatialnode/superstac/main/docs/assets/superstac-logo.svg" alt="SuperSTAC logo" width="480">
  </picture>
</p>

Search STAC catalogs from Python scripts and notebooks. Use `Client` for regular
Python code or `AsyncClient` for asyncio. Results are ordinary dictionaries.

SuperSTAC is **alpha software**; APIs may change before v1.0.

## Install

Use Python 3.9 or newer:

```bash
python -m pip install superstac
```

For a source build, follow the
[installation guide](https://spatialnode.com/superstac/docs/start/installation).

## Search catalogs

```python
from superstac import Client

client = Client(catalogs=[
    {"id": "earth-search", "url": "https://earth-search.aws.element84.com/v1"},
    {"id": "microsoft", "url": "https://planetarycomputer.microsoft.com/api/stac/v1"},
])
try:
    search = client.search(
        collections=["sentinel-2-l2a"],
        bbox=[6.0, 49.0, 7.0, 50.0],
        datetime="2024-01-01T00:00:00Z/2024-01-31T23:59:59Z",
        limit=5,
    )
    for item in search.items():
        print(item["id"], item["properties"]["datetime"])
    print(search.metadata)
    geojson = search.to_geojson()
finally:
    client.shutdown()
```

`limit=5` allows up to five items from each catalog. `search.matched()` counts the
items returned, after duplicates are removed if enabled. Check
`search.metadata["failures"]` even when the search returns items: some catalogs
may have failed.

To use one catalog, create the client with
`Client.open("https://earth-search.aws.element84.com/v1")`. To share a configuration
with the CLI or Rust, use `Client.from_yaml("superstac.yml")` and the
[complete sample file](https://spatialnode.com/superstac/examples/superstac.yml).

[Run the notebook](https://spatialnode.com/superstac/docs/python/notebook) to see
scene footprints on a map and download the results.

## Use asyncio

```python
import asyncio
from superstac import AsyncClient

async def main():
    client = await AsyncClient.open("https://earth-search.aws.element84.com/v1")
    try:
        search = await client.search(collections=["sentinel-2-l2a"], limit=5)
        for item in search.items():
            print(item["id"])
        print(search.metadata)
    finally:
        await client.shutdown()

asyncio.run(main())
```

In a notebook, use `await main()` instead of `asyncio.run(main())`.
`AsyncClient(...)` and `AsyncClient.from_yaml(...)` do not need `await`.

## Search saved metadata

Python wheels include GeoParquet support. Save metadata for your study area, then
reuse it across queries. With an inventory in `./data`:

```python
client = Client.from_yaml("superstac.yml", mode="snapshot", dataset="./data")
```

Use `mode="auto"` to fall back to live catalogs when saved coverage is missing or
older than one day. Change that age limit with `max_snapshot_age_seconds`.
Live results are not saved automatically, and imagery still comes from the provider.

The [GeoParquet guide](https://spatialnode.com/superstac/docs/guides/geoparquet)
covers saving, updating, and cleaning up an inventory. You can also
[try the notebook](https://spatialnode.com/superstac/docs/python/geoparquet-notebook).

## Switching from an older client

Version 0.2 replaced the old Python implementation. If you used `0.1.0a2`, start
with the [Python guide](https://spatialnode.com/superstac/docs/python/overview).

Some methods resemble pystac-client, but items are dictionaries rather than
`pystac.Item` objects. `matched()` counts returned items, and `to_geojson()` exports
a FeatureCollection. Custom authentication hooks and the full pystac-client API
are not supported. See the [API reference](https://spatialnode.com/superstac/docs/python/api)
for available methods.

## License

MIT.
