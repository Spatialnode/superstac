---
title: Results and provenance
description: Interpret item wrappers, metadata counts, partial failures, and deduplication.
---

## Rust and CLI response

The following is a schematic response; the `item` dictionary is abbreviated, not a complete STAC Item.

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

`catalog_id` is the catalog whose item body was retained. `seen_in` tracks catalogs returning the same item ID. Deduplication does not combine asset dictionaries or resolve which provider has the best data.

## Metadata fields

| Field | Meaning |
| --- | --- |
| `catalogs_queried` | Number selected for execution, after health/source filtering. |
| `catalogs_succeeded` | Selected catalogs that completed successfully. |
| `catalogs_failed` | Selected catalogs that failed. |
| `total_items` | Items in the final response, after optional deduplication. |
| `duplicates_removed` | Number of collapsed item records. |
| `failures` | List of `{catalog_id, reason}` for failed searches. |
| `unsupported_collections` | Requested canonical IDs that no candidate catalog was known to serve, reported conservatively when collection knowledge is complete. |

A catalog excluded for health or capability reasons is not necessarily represented in `failures`. Compare metadata with your configured registry when diagnosing missing sources.

## Python representation

`search.items()` unwraps the item bodies into Python dictionaries. The per-item `catalog_id` and `seen_in` fields are not currently exposed by the Python `Search` object. `search.metadata` exposes the run-level fields above.

`search.to_geojson()` returns `{ "type": "FeatureCollection", "features": [...] }`. It omits the Rust provenance wrappers and run metadata. Store `search.metadata` separately if you need a record of search failures.

## Deduplication identity

Deduplication compares `Item.id` across the entire response, without including the collection ID. Disable it if your sources reuse item IDs for unrelated records:

```python
from superstac import Client
client = Client(settings={"deduplicate_items": False})
```

Repeated observations with different IDs are not recognized as duplicates. Output order and the retained provider are not a stable ranking.
