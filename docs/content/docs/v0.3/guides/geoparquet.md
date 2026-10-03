---
title: GeoParquet inventories
description: Save STAC metadata for your study area, search it locally, and update it when needed.
---

If you often search the same region, save its STAC metadata to GeoParquet and reuse
it across queries. You can change dates and filters within the saved coverage
without fetching the metadata again. Imagery and previews still come from the
provider.

For a runnable Python example, [open the notebook](/docs/v0.3/python/geoparquet-notebook).
The CLI examples below use a catalog named `earth-search` in `superstac.yml` and
its configured collection name, `sentinel-2-l2a`.

For setup, see [installation](/docs/v0.3/start/installation).

## Save your first inventory

Start with a small area and date range. Downloading an entire catalog can take a
long time and use substantial storage.

```sh
superstac ingest --catalog earth-search --name madrid-feb \
  --collection sentinel-2-l2a --datetime 2025-02-01/2025-03-01 \
  --bbox=-4.5,39.5,-3,41 --output ./data --max-dataset-mib 1024
```

This saves metadata under `./data` and shows progress as pages arrive. A percentage
and ETA appear when the provider supplies a total. Use `--no-progress` to hide
progress; JSON output stays on stdout and progress goes to stderr.

`madrid-feb` names this saved area and date range, called a **scope**. Use another
name to add a different area or month to the same dataset. Repeat `--catalog` to
include more catalogs. Ingestion only runs when you request it.

If the download stops, repeat the command with `--resume`. Keep the name, filters,
and page size unchanged. If a provider’s continuation token has expired, restart
without `--resume`. Resuming an already complete scope does not refresh it.

## Search saved metadata

```sh
superstac --mode snapshot --dataset ./data --json search \
  --collection sentinel-2-l2a --datetime 2025-02-10/2025-02-15 \
  --bbox=-4,40,-3.5,40.5 --limit 20
```

Your search can differ from the ingestion query, as long as the saved collections,
area, and dates cover it. Several scopes can cover a query together, but gaps count
as missing coverage. Snapshot mode never contacts providers; check per-catalog
response metadata for coverage failures.

Add `--sortby=-datetime` for newest first. Sorting applies within each catalog,
not across the combined response. Collection discovery describes the saved data,
rather than the provider’s entire catalog.

## Fall back to live catalogs

Use `auto` when you want local searches where possible and live searches otherwise:

```sh
superstac --mode auto --dataset ./data --max-snapshot-age-seconds 86400 \
  search --collection sentinel-2-l2a --datetime 2025-02-10/2025-02-15 \
  --bbox=-4,40,-3.5,40.5 --limit 20
```

| Mode | Where the results come from |
| --- | --- |
| `snapshot` | Saved data only, with no age limit. |
| `auto` | Complete, fresh local coverage; otherwise the catalog’s API. |
| `live` | The catalog’s API. |

Auto mode defaults to a maximum age of one day, measured from the start of
ingestion. It also falls back if the dataset is missing, unreadable, or belongs to
a different source URL. A valid local search with no matches stays local.

**Live results are not saved automatically.** Run `ingest` to add coverage you want
to reuse. Auto mode reads the latest manifest on each search; reopen snapshot
clients after updating their dataset.

Without `--mode`, supplying a dataset or file selects snapshot mode; otherwise
SuperSTAC uses live mode. To search an existing standalone file:

```sh
superstac --geoparquet earth-search=./items.parquet search --limit 20
```

Repeat `--geoparquet CATALOG=PATH` for more catalogs. Standalone files do not carry
the managed dataset’s coverage and freshness guarantees.

## Use Python

```python
from superstac import Client

client = Client.from_yaml("superstac.yml")
client.ingest(
    "earth-search", "./data", name="madrid-feb",
    collections=["sentinel-2-l2a"], datetime="2025-02-01/2025-03-01",
    bbox=[-4.5, 39.5, -3, 41],
    progress=lambda event: print(event["phase"], event["items_saved"]),
)
local = Client.from_yaml("superstac.yml", mode="snapshot", dataset="./data")
results = local.search(
    collections=["sentinel-2-l2a"], datetime="2025-02-10/2025-02-15",
    bbox=[-4, 40, -3.5, 40.5], sortby=["-datetime"], limit=20,
)
```

Use `mode="auto"` and `max_snapshot_age_seconds=86400` for live fallback.
`AsyncClient` supports the same options, with awaitable `ingest`, search, and
maintenance methods. Constructors and `from_yaml()` remain synchronous.

Progress callbacks run on a worker thread and should return promptly. Callback
exceptions are reported without stopping ingestion. See the
[Python API](/docs/v0.3/python/api) for method signatures.

## Update an inventory

Rerun ingestion with the same name to replace that scope. Other scopes remain
unchanged. Changing its filters replaces its previous coverage too.

To fetch only a recent acquisition window, use `--incremental-since`:

```sh
superstac ingest --catalog earth-search --name madrid-feb \
  --collection sentinel-2-l2a --datetime 2025-02-01/2025-03-05 \
  --bbox=-4.5,39.5,-3,41 --output ./data \
  --incremental-since 2025-02-25T00:00:00Z
```

Keep the existing scope’s source, collections, area, and start date. You may extend
the end date; the update window must overlap the previous coverage without a gap.
New records replace saved copies with the same collection and item ID.

This checks acquisition dates, not the provider’s change history. It cannot detect
deletions or changes outside the window. Run a full refresh without
`--incremental-since` when you need to reconcile those changes. Incremental updates
keep the original freshness timestamp because older records haven’t been rechecked.

## Manage disk space

Ingestion has a default **1,024 MiB storage limit** for Parquet files, including old
files and unfinished downloads. Set `--max-dataset-mib` to raise it, or `0` to remove
the limit. If ingestion hits the limit, raise it and resume. Metadata and filesystem
overhead are outside the limit.

A dataset can contain many Parquet files; you search the dataset directory as one
inventory. Use `--items-per-file` to adjust the approximate rows per file.
`--page-size` controls each provider request, not the total number of items saved.

Old files are kept until you explicitly clean them up. Allow room for both old and
new files during a refresh or compaction.

```sh
superstac compact --output ./data --items-per-file 10000 --max-dataset-mib 2048
superstac cleanup --output ./data              # preview what can be removed
superstac cleanup --output ./data --apply      # remove unreferenced Parquet files
```

Compaction combines updates and removes duplicate records while preserving coverage
and freshness. If it fails, the previous inventory remains readable. Cleanup
protects files used by the current manifest and saved checkpoints. Close snapshot
clients before applying cleanup; active readers or writers prevent it from running.
`shutdown()` stops monitoring but does not release a client’s snapshot lock.

## Rust

Enable the engine’s `geoparquet` feature. Use `ingest_catalog` with `IngestOptions`
to save data, `SuperSTACEngine::from_dataset` for local search, or
`SuperSTACEngine::automatic(storage, path, max_age)` for live fallback.

The engine also exports `compact_dataset` and `cleanup_dataset`.
`IngestOptions::progress` accepts an `Arc<dyn Fn(&IngestProgress) + Send + Sync>`;
callbacks should return promptly and not panic. Datasets currently use local
storage, not remote object stores.

## Performance and version information

Selective queries can skip irrelevant Parquet row groups. Broad queries and sorting
read more data; searches spanning multiple scopes or uncompacted updates also do
more work. Compaction can help after incremental updates. Deduplication uses memory
proportional to the unique items scanned. See the [benchmark](/docs/v0.3/guides/benchmarks)
for measurements and how to reproduce them.

Newly written files record the SuperSTAC version automatically, including after
compaction. Existing files keep their metadata until rewritten. To inspect a file:

```python
import json
import pyarrow.parquet as pq

footer = pq.read_metadata("path/to/part.parquet")
print(footer.created_by)
print(json.loads(footer.metadata[b"superstac"]))
```

For bug reports, include `superstac --version` or Python’s `superstac.__version__`.
Search diagnostics also include `metadata.superstac_version`.

Reusing an inventory from an older release? See the [upgrade notes](/docs/v0.3/releases).
