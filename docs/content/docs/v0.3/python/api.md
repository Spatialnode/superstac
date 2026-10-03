---
title: Python API reference
description: Methods for creating clients, managing catalogs, searching, and reading results.
---

Import the public types with `from superstac import Client, AsyncClient`. Both clients accept `storage="memory"`; other storage backends are rejected by the current Python bindings.

## Constructors

```text
Client(config=None, *, catalogs=None, providers=None, settings=None,
       storage="memory", mode="live", dataset=None,
       max_snapshot_age_seconds=86400)
Client.open(url, *, id=None, storage="memory", mode="live",
            dataset=None, max_snapshot_age_seconds=86400)
Client.from_yaml(yaml_path, *, storage="memory", mode="live",
                 dataset=None, max_snapshot_age_seconds=86400)
```

`AsyncClient` has the same signatures. Await only `AsyncClient.open(...)`; its constructor and `from_yaml()` are synchronous.

`config` is a dictionary with optional `catalogs`, `providers`, and `settings` entries. Supplying it takes precedence over the separate constructor keywords. Without an explicit ID, `open()` derives one from the first dot-separated hostname label.

## Lifecycle and settings

| Method | Behavior |
| --- | --- |
| `start()` | Initial health checks and collection discovery; no-op while started. |
| `shutdown()` | Cancel monitors and mark the engine unstarted. |
| `get_settings()` | Return the stored settings dictionary. |
| `update_settings(update)` | Apply supported non-null fields from a dictionary. |

On `AsyncClient`, await `start()` and `shutdown()`. Read [health limitations](/docs/v0.3/guides/resilience/) before relying on global monitoring settings.

## Catalog registry

| Method | Input / output |
| --- | --- |
| `add_catalog(catalog, *, provider=None)` | Add one catalog dictionary; return created catalog. |
| `add_catalogs(catalogs)` | Add a list of dictionaries, honoring each `provider` field. |
| `get_catalog(id)` | Return one catalog dictionary. |
| `list_catalogs()` | Return registered catalogs. |
| `update_catalog(id, update)` | Update supported catalog fields. |
| `delete_catalog(id)` | Remove a catalog. |

Always include `id` and `url` when creating a catalog. `update_catalog` accepts `provider`, `title`, `description`, `url`, and `settings`. Set aliases when you create the catalog; `update_catalog()` cannot change them. Include the existing title and description in an update if you want to keep them, because omitted values can be cleared.

## Provider registry

`add_provider(provider)`, `add_providers(providers)`, `get_provider(id)`, `list_providers()`, `update_provider(id, update)`, and `delete_provider(id)` operate on descriptive provider records. Register a provider before linking catalogs to it.

These registry methods remain synchronous on `AsyncClient`.

## Discovery

| Method | Result |
| --- | --- |
| `list_collections()` | List of `{"id": collection_id, "catalogs": [catalog_id, ...]}`. |
| `collections_by_catalog()` | Dictionary of catalog IDs to collection ID lists. |
| `catalogs_supporting(collection_id)` | Catalog IDs advertising the collection. |
| `describe_collection(catalog_id, collection_id)` | Full collection dictionary or `None` for a 404. |
| `get_collection(id)` | First matching collection document; raises `KeyError` if none is found. |
| `get_collections()` | Full documents for discovered collections; potentially many requests. |

Await all discovery methods on `AsyncClient`.

## Search

```python
client.search(
    collections=["sentinel-2-l2a"],
    bbox=[6.0, 49.0, 7.0, 50.0],
    datetime="2024-01-01T00:00:00Z/2024-01-31T23:59:59Z",
    limit=10,
)
```

Pass `collections` explicitly, using `[]` for all collections. See [search parameters](/docs/v0.3/reference/search/) for `ids`, `intersects`, how limits work, and unsupported parameters. The return value is a `Search` object; await the call on `AsyncClient`.

## Search results

`items()`, `matched()`, `to_geojson()`, `item_collection_as_dict()`, `len(search)`, and the `metadata` property are synchronous. See [result fields](/docs/v0.3/reference/results/).

Python returns the STAC item dictionaries without Rust’s `catalog_id` and `seen_in` fields. Use metadata to check failures and counts for the whole search; it does not identify the source of each item.

## Saved GeoParquet inventories

Published Python wheels include these methods. Check a custom build with `from superstac import geoparquet_available`.

| Constructor option | Use |
| --- | --- |
| `mode="live"` | Query catalog APIs; the default. |
| `mode="snapshot", dataset="./data"` | Search saved metadata only. |
| `mode="auto", dataset="./data"` | Use complete, fresh saved coverage or fall back to the API. |
| `max_snapshot_age_seconds=86400` | Maximum saved-data age in auto mode; measured from ingestion start. |

```text
client.ingest(catalog_id, output, *, collections=[], bbox=None,
              datetime=None, name=None, resume=False,
              incremental_since=None, page_size=500,
              items_per_file=10000, max_dataset_mib=1024,
              timeout_seconds=60, all=False, progress=None)
client.compact_dataset(dataset, *, items_per_file=10000,
                       max_dataset_mib=1024)
client.cleanup_dataset(dataset, *, apply=False)
```

All three methods return dictionaries. Await them on `AsyncClient`. Ingestion needs collection, spatial, or temporal scope unless you explicitly opt into `all=True`. `page_size` controls provider requests, not the total number of records saved.

`progress` receives a dictionary with fields such as `phase` and `items_saved`. It runs on a worker thread: keep it short and avoid changing notebook or browser widgets directly from it.

Cleanup previews removable files by default; `apply=True` removes unreferenced files. Release snapshot clients before cleanup or compaction. `shutdown()` stops background tasks but keeps the client’s dataset lock until the object is released.

See [the complete inventory workflow](/docs/v0.3/guides/geoparquet#use-python) for ingestion, local search, and cleanup.

## Errors

Invalid Python input generally raises `ValueError`. Engine errors are mapped to `RuntimeError`; `get_collection()` uses `KeyError` for a missing collection. A search can also return normally with catalog failures listed in `search.metadata["failures"]`. Check that list even when no exception is raised.
