---
title: Results and provenance
description: Read search results, check failed catalogs, and understand how duplicate items are handled.
---

## JavaScript, Rust, and CLI response

A response contains items and metadata about the search. This example shortens the STAC item to show the surrounding fields:

```json
{
  "items": [
    {
      "catalog_id": "earth-search",
      "item": {"id": "example-scene", "collection": "sentinel-2-l2a"},
      "seen_in": ["earth-search", "microsoft"]
    }
  ],
  "metadata": {
    "catalogs_queried": 2,
    "catalogs_succeeded": 2,
    "catalogs_failed": 0,
    "total_items": 1,
    "duplicates_removed": 1,
    "failures": [],
    "unsupported_collections": []
  }
}
```

`catalog_id` tells you which catalog supplied the retained item. `seen_in` lists catalogs that returned the same item ID. SuperSTAC keeps one copy with its original assets; it does not choose the best provider or merge their assets.

## Metadata fields

| Field | Meaning |
| --- | --- |
| `superstac_version` | Version of the engine that produced the response. |
| `catalogs_queried` | Number selected for execution, after health/source filtering. |
| `catalogs_succeeded` | Selected catalogs that completed successfully. |
| `catalogs_failed` | Selected catalogs that failed. |
| `total_items` | Items in the final response, after optional deduplication. |
| `duplicates_removed` | Number of collapsed item records. |
| `failures` | List of `{catalog_id, reason}` for failed searches. |
| `unsupported_collections` | Requested canonical IDs that no candidate catalog was known to serve, reported conservatively when collection knowledge is complete. |

In the native engine, a catalog skipped because it is unhealthy or does not offer the requested collection may be absent from `failures`. If a source is missing, compare the metadata with the catalogs you registered.

## Python representation

`search.items()` unwraps the item bodies into Python dictionaries. The per-item `catalog_id` and `seen_in` fields are not currently exposed by the Python `Search` object. `search.metadata` exposes the run-level fields above.

`search.to_geojson()` returns `{ "type": "FeatureCollection", "features": [...] }`. It omits the Rust provenance wrappers and run metadata. Store `search.metadata` separately if you need a record of search failures.

## How duplicates are identified

Deduplication compares `Item.id` across the entire response, without including the collection ID. Disable it if your sources reuse item IDs for unrelated records:

```python
from superstac import Client
client = Client(settings={"deduplicate_items": False})
```

The same scene with different IDs remains as separate items. Result order and the provider whose copy is kept can vary between searches.
