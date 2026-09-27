---
title: How federation works
description: Follow a search from one request to multiple catalogs and back to a combined result.
---

SuperSTAC is a client-side search engine. It sends requests to STAC API catalogs you register; it does not maintain a global index or copy imagery into its own storage.

## The vocabulary

| Term | Meaning |
| --- | --- |
| Catalog | A registered STAC API endpoint with a local ID and URL. |
| Provider | Optional descriptive grouping for catalogs. |
| Collection | A dataset within a catalog, such as `sentinel-2-l2a`. |
| Item | A STAC record describing a scene or other geospatial asset. |
| Asset | A named link within an item, such as a spectral band or thumbnail. |
| Canonical name | The collection ID or asset key your application uses across providers. |

## From request to response

1. **Start and inspect.** The engine checks catalog health and discovers `/collections` on healthy catalogs.
2. **Choose sources.** It normally excludes unhealthy catalogs and catalogs known not to serve any requested collection. If collection discovery failed, the unknown catalog can still be queried.
3. **Translate names.** Each catalog's collection aliases translate your canonical query into local names.
4. **Search concurrently.** The executor bounds concurrent catalog searches, applies timeouts and retries, and follows upstream pagination up to a per-catalog cap.
5. **Normalize and combine.** Asset keys and collection IDs can be normalized. Items with the same ID can be collapsed, and failures are included in metadata.

An empty `collections` list removes the collection filter; it does not select a default dataset.

## What federation does not guarantee

Different providers may index different dates, use different item IDs for the same scene, or expose different assets. Searching two catalogs does not guarantee identical coverage or double the number of unique items.

Deduplication uses **item ID only**. Equal IDs in unrelated collections can be collapsed; different IDs for the same physical scene remain separate. The first encountered item body is retained, without merging its asset dictionaries. Concurrent completion and storage ordering mean you should not rely on a particular provider winning.

There is no global sort or stable pagination cursor across catalogs. See [search parameters](/docs/reference/search/) and [results](/docs/reference/results/) for exact semantics.
