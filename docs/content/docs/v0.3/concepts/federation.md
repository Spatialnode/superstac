---
title: How federation works
description: Follow a search from one request to multiple catalogs and back to a combined result.
---

Federation means searching several catalogs through one client. In live mode, SuperSTAC sends your query to the STAC APIs you register and combines their responses. You can also [save metadata locally](/docs/v0.3/guides/geoparquet) for repeated searches. Imagery stays with the provider.

## Terms used in these docs

| Term | Meaning |
| --- | --- |
| Catalog | A registered STAC API endpoint with a local ID and URL. |
| Provider | Optional descriptive grouping for catalogs. |
| Collection | A dataset within a catalog, such as `sentinel-2-l2a`. |
| Item | A STAC record describing a scene or other geospatial asset. |
| Asset | A named link within an item, such as a spectral band or thumbnail. |
| Canonical name | The collection ID or asset key your application uses across providers. |

## From request to response

The Python, Rust, and CLI live-search path follows these steps:

1. **Start and inspect.** The engine checks catalog health and discovers `/collections` on healthy catalogs.
2. **Choose sources.** It normally excludes unhealthy catalogs and catalogs known not to serve any requested collection. If collection discovery failed, the unknown catalog can still be queried.
3. **Translate names.** Each catalog's collection aliases translate your canonical query into local names.
4. **Search in parallel.** SuperSTAC limits how many catalogs it queries at once, retries failed requests, and reads result pages up to the limit for each catalog.
5. **Combine the results.** SuperSTAC can rename collections and assets using your aliases and remove duplicate item IDs. The response also lists any catalog failures.

The browser binding sends searches directly to the catalogs you configure. It supports aliases, pagination, retries, and deduplication, but does not run background health checks or choose catalogs from discovered capabilities.

An empty `collections` list removes the collection filter; it does not select a default dataset.

## Coverage and duplicate items

Different providers may index different dates, use different item IDs for the same scene, or expose different assets. Searching two catalogs does not guarantee identical coverage or double the number of unique items.

Deduplication uses **item ID only**. Equal IDs in unrelated collections can be collapsed; different IDs for the same physical scene remain separate. SuperSTAC keeps the first item it encounters and leaves its assets as they are. Which provider supplies that item can vary between searches.

There is no global sort or stable pagination cursor across catalogs. See [search parameters](/docs/v0.3/reference/search/) and [results](/docs/v0.3/reference/results/) for details.
