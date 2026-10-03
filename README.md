# SuperSTAC

[![License](https://img.shields.io/badge/license-MIT-green.svg)](https://github.com/spatialnode/superstac/blob/main/LICENSE)
[![crates.io](https://img.shields.io/crates/v/superstac-search.svg)](https://crates.io/crates/superstac-search)
[![Documentation](https://img.shields.io/badge/docs-SuperSTAC-blue)](https://spatialnode.com/superstac)
[![docs.rs](https://docs.rs/superstac-search/badge.svg)](https://docs.rs/superstac-search)

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/spatialnode/superstac/main/docs/assets/superstac-logo-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/spatialnode/superstac/main/docs/assets/superstac-logo.svg">
    <img src="https://raw.githubusercontent.com/spatialnode/superstac/main/docs/assets/superstac-logo.svg" alt="SuperSTAC logo" width="480">
  </picture>
</p>

**Many catalogs. One search.**

Search [STAC](https://stacspec.org/) catalogs from Python, Rust, or the command line.
Query Earth Search, Microsoft Planetary Computer, and other catalogs together,
then work with the results in one place.

SuperSTAC can use your preferred collection and band names, remove duplicate item
IDs, and report failed catalogs. If you often search the same area, save its
metadata to GeoParquet and search it locally. Imagery stays with the provider.

SuperSTAC is **alpha software**. APIs and configuration may change before v1.0.

[Documentation](https://spatialnode.com/superstac) ·
[What’s new in v0.3](https://spatialnode.com/superstac/docs/releases) ·
[Try a notebook](https://spatialnode.com/superstac/docs/python/notebook)

## Start with Python

Use Python 3.9 or newer:

```bash
python -m pip install superstac
```

Search Sentinel-2 imagery in two catalogs:

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
        print(item["id"], item["collection"])
    print(search.metadata)
finally:
    client.shutdown()
```

The limit applies to each catalog, so this can return up to ten items before
duplicate IDs are removed. Check `search.metadata["failures"]` for any catalogs
that failed. Results are Python dictionaries; `search.to_geojson()` returns a
GeoJSON FeatureCollection.

See the [Python guide](https://spatialnode.com/superstac/docs/python/overview)
for more examples, including asyncio.

## Use the command line

Build with Rust 1.88 or newer:

```bash
git clone https://github.com/spatialnode/superstac
cd superstac
cargo install --path crates/cli --features geoparquet
cp docs/public/examples/superstac.yml ./superstac.yml
```

The sample configuration includes Earth Search and Microsoft Planetary Computer.
Edit it to use your own catalogs, then run:

```bash
# List available collections
superstac collections

# Search both catalogs
superstac search -c sentinel-2-l2a -b 6.0,49.0,7.0,50.0 -l 5

# Read search metadata as JSON
superstac --json search -c sentinel-2-l2a -l 5 | jq '.metadata'
```

Run `superstac --help` for commands and options. The
[configuration reference](https://spatialnode.com/superstac/docs/reference/configuration)
explains aliases, timeouts, and retries.

## Browser / WebAssembly

Build the browser binding to search live STAC catalogs from JavaScript or TypeScript.
See the [WASM guide](crates/wasm/README.md) for build instructions, API usage,
a browser example, and browser-specific limitations. The docs include a
[live playground](https://spatialnode.com/superstac/docs/wasm/playground)
and a [JavaScript guide](https://spatialnode.com/superstac/docs/wasm/overview).

## Configuration
## Save metadata for repeated searches

GeoParquet inventories let you reuse metadata for an area and date range. Search
saved data only, or use automatic mode to query live catalogs when the saved data
is too old or does not cover your query. Python wheels include GeoParquet support.

[Try the GeoParquet notebook](https://colab.research.google.com/drive/1JIx8j0Vi7jypKe5Q6klgCzYQMFHMaKGc?usp=sharing) ·
[Read the guide](https://spatialnode.com/superstac/docs/guides/geoparquet) ·
[See the benchmark](https://spatialnode.com/superstac/docs/guides/benchmarks)

## Use Rust

```bash
cargo add superstac-core superstac-config superstac-search superstac-engine
cargo add tokio --features macros,rt-multi-thread
```

The [Rust guide](https://spatialnode.com/superstac/docs/rust/overview) has a complete
search example. Use `superstac-engine` to manage catalogs and run searches;
`superstac-core`, `superstac-config`, and `superstac-search` provide the types,
configuration loading, and search queries.

## Help and contributing

For unexpected results, start with
[troubleshooting](https://spatialnode.com/superstac/docs/reference/troubleshooting).
To report a bug, [open an issue](https://github.com/spatialnode/superstac/issues)
with your version, query, and failure metadata. Remove credentials from any
configuration you share.

See [ROADMAP.MD](ROADMAP.MD) for planned work.

## License

[MIT](LICENSE).
