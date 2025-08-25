# SuperSTAC 

[![PyPI version](https://img.shields.io/pypi/v/superstac.svg)](https://pypi.org/project/superstac/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](./LICENSE)

**SuperSTAC** is a Python library (with planned Rust backend) for **high-availability satellite imagery retrieval**.  
Instead of relying on a single STAC endpoint (e.g., Sentinel from Element84), SuperSTAC can query **multiple catalogs** and automatically fall back to alternatives when a source is missing data or unavailable.

⚠️ **Note:** This is an **early work-in-progress**. The initial release is to start iterating in public. Expect breaking changes.

---

## Features (planned)

- Query multiple STAC catalogs through a single unified API.
- Automatic fallback when a catalog has no data or is down.
- Configurable authentication for protected catalogs.
- Resolution & band matching across heterogeneous catalogs.
- CLI and Python API for flexible workflows.
- Optional LLM-assisted natural language queries.
- Rust backend (planned)

---

## Installation

```bash
pip install superstac
```

## Configuration

SuperSTAC loads its catalog configuration from a YAML file, typically referenced via the environment variable `SUPERSTAC_CATALOG_CONFIG`.

Example `.superstac.yml`:

```bash
catalogs:
  Element84 Sentinel:
    url: https://earth-search.aws.element84.com/v0
  Planet:
    url: https://api.planet.com/stac/v1
    auth:
      type: basic
      username: youruser
      password: yourpass
  Microsoft PC:
    url: https://planetarycomputer.microsoft.com/api/stac/v1
    auth:
      type: bearer
      token: "YOUR_MICROSOFT_PC_TOKEN"
```
See [superstac/.superstac.yml](superstac/.superstac.yml) for an example config file.


## Usage (very early draft)

```python
  from superstac import get_catalog_registry, federated_search_async

  cr = get_catalog_registry()
  cr.load_catalogs_from_config()

  print("\nRunning asynchronous federated_search_async...")
  start_async = time.perf_counter()
  results_async = asyncio.run(
      federated_search_async(
          registry=cr,
          collections=["sentinel-2-l2a"],
          bbox=[6.0, 49.0, 7.0, 50.0],
          datetime="2024-01-01/2024-01-31",
          query={"eo:cloud_cover": {"lt": 20}},
          sortby=[{"field": "properties.datetime", "direction": "desc"}],
      )
  )
  end_async = time.perf_counter()
  print(
      f"Asynchronous search found {len(results_async)} items in {end_async - start_async:.2f} seconds."
  )

  for x in results_async:
      print(x.self_href)
```

Also see [main.py](./main.py).

## Development Status / Roadmap

Planned enhancements:
 - Authentication configuration & documentation
 - Retry logic
 - Result modifiers
 - Catalog refresh & health checks
 - Latency tracking and fallback ranking
 - Band matching across heterogeneous catalogs
 - CLI tool
 - Example notebooks (illegal mining detection, disaster response, LLM-assisted search)


## License

MIT License. See [LICENSE](LICENSE).

Feedback, issues, and contributions are welcome! This package is at a very early stage, so opening issues for missing features or edge cases will directly shape the roadmap.