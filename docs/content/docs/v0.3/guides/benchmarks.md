---
title: Local search benchmark
description: How much does skipping irrelevant Parquet data help a local search?
---

For a selective time query over **100,000 synthetic STAC items**, local search took
**4.2 ms with row-group pruning**, compared with **411.3 ms without it** in this run.
Pruning means skipping blocks of data that cannot match the query. Both searches
returned the same records. This compares two local scans, not local search against
a provider API; downloading the inventory is excluded.

![Median search latency: 411.3 milliseconds without row-group pruning and 4.2 milliseconds with pruning, over 100,000 synthetic items.](../../../../public/versions/v0.3/benchmarks/geoparquet-v0.3.svg)

| Measurement | Value |
| --- | --- |
| Fixture | 100,000 time-ordered point items, one collection |
| File size | 858,390 bytes; minimal, highly compressible metadata |
| Layout | 1,024 rows per group |
| Query | The last 20 records by acquisition-time filter; no sorting |
| Statistic | Median of five sequential runs; OS cache may be warm |
| Environment | Apple M3 Pro, 36 GiB RAM, macOS 15.5, Rust 1.88.0 |
| Build | SuperSTAC v0.3 development code, optimized release profile |
| Recorded | October 2, 2026 |

The baseline decodes row groups sequentially until the result limit. The optimized
path consults Parquet statistics and skips groups that cannot match. The benchmark
asserts that the returned IDs are identical on every repetition.

## When it helps

Skipping data helps most when a query selects a small part of a time-ordered
inventory. This synthetic dataset is a favorable case. Broader queries, larger
records, and sorting take more work, so expect different timings on your own data.

Searches across multiple scopes or uncompacted incremental updates currently scan
without pruning to handle duplicate records correctly. Compacting an updated scope
lets it use pruning again. Deduplication also needs memory for the item IDs scanned.

## Reproduce it

```sh
cargo test -p superstac-geoparquet --release benchmark_scoped_search -- --ignored --nocapture
```

The fixture is generated locally and removed afterward. There are no provider
requests. Output includes the item count, file bytes, and both median latencies.
Increase `SUPERSTAC_BENCH_ITEMS` only when you have enough memory: fixture generation
currently materializes the synthetic items, unlike the bounded ingestion buffers.

[Download measurements as JSON](/superstac/versions/v0.3/benchmarks/geoparquet-v0.3.json) ·
[Download the figure](/superstac/versions/v0.3/benchmarks/geoparquet-v0.3.png) ·
[Plotting script](https://github.com/spatialnode/superstac/blob/main/scripts/plot-geoparquet-benchmark.py)

Start with the [GeoParquet notebook](/docs/v0.3/python/geoparquet-notebook) or read the
[dataset guide](/docs/v0.3/guides/geoparquet) before measuring your own inventory.
