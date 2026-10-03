---
title: Use SuperSTAC in the browser
description: Search STAC catalogs from JavaScript or TypeScript with the WebAssembly binding.
---

The browser binding runs SuperSTAC's Rust search logic in your JavaScript app. Register catalogs, send one query, and receive combined results with source information.

[Try the playground](/docs/wasm/playground) to see it without installing anything.

## Build the package

The browser package is currently built from source; it has not been published to npm. You need Rust 1.88 or newer.

From the repository root:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.13.1 --locked --jobs 1
wasm-pack build crates/wasm --target web --release --locked --jobs 1
```

The generated package is in `crates/wasm/pkg`. Serve `superstac_wasm.js` and `superstac_wasm_bg.wasm` together over HTTP or HTTPS. The `.d.ts` file supplies TypeScript types. Build commands use one Cargo worker to limit resource use.

## Search two catalogs

```js
import init, { SuperSTAC } from './pkg/superstac_wasm.js';

await init();
const client = new SuperSTAC({
  catalogs: [
    { id: 'earth-search', url: 'https://earth-search.aws.element84.com/v1' },
    { id: 'microsoft', url: 'https://planetarycomputer.microsoft.com/api/stac/v1' },
  ],
});

try {
  const result = await client.search({
    collections: ['sentinel-2-l2a'],
    bbox: [6, 45, 7, 46], // West, south, east, north, in degrees.
    datetime: '2024-06-01T00:00:00Z/2024-06-30T23:59:59Z',
    limit: 5,
  });

  for (const { item, seen_in } of result.items) {
    console.log(item.id, seen_in);
  }
  console.log(result.metadata.failures);
} finally {
  client.free(); // After all pending calls have finished.
}
```

Initialization loads the WASM module once. `search()` returns a Promise. `limit` is per catalog, defaults to 10, and is capped at 1,000 by default. Omit `collections` to search without a collection filter. Other query fields are `ids`, `intersects`, and `sortby`; use either `bbox` or `intersects`, not both.

## Understand the response

`result.items` contains entries with:

- `item`: the STAC Feature, including properties, assets, and links.
- `catalog_id`: the catalog whose copy was kept.
- `seen_in`: all catalogs that returned that item ID.

`result.metadata` includes counts, duplicate records removed, and per-catalog failures. A request can resolve successfully even if every catalog failed, so check `metadata.failures` as well as `items`.

Duplicate detection uses the item ID. Two records of the same acquisition with different IDs are not automatically combined. The merged results are not globally sorted.

To make a GeoJSON FeatureCollection:

```js
const geojson = {
  type: 'FeatureCollection',
  features: result.items.map(({ item }) => item),
};
```

## Discover collections

```js
const catalogs = await client.listCollections();
for (const catalog of catalogs) {
  console.log(catalog.catalog_id, catalog.collections, catalog.error);
}
```

Each collection entry includes its `canonical_id` and original `collection` metadata. Discovery follows pagination, returns up to 10,000 collections per catalog, and reports failures separately. It does not change which catalogs future searches query.

## Configure the client

All settings below are optional. These are the defaults:

```js
const client = new SuperSTAC({
  catalogs: [
    { id: 'earth-search', url: 'https://earth-search.aws.element84.com/v1' },
  ],
  settings: {
    deduplicate_items: true,
    unify_response: true,
    max_concurrent_catalogs: 8,
    per_catalog_timeout_seconds: 30,
    max_retry_attempts: 2,
    max_items_per_catalog: 1000,
  },
});
```

`max_retry_attempts` includes the first attempt. Timeouts cover one catalog's paginated search attempt. Failed attempts discard their partial results. Background tabs can delay browser timers.

Catalogs can also include `collection_aliases` and `asset_aliases`, using the [same canonical-to-provider mappings](/docs/guides/aliases) as the other interfaces.

## Browser requirements

Catalogs must allow browser requests through CORS. WASM cannot bypass that restriction; an application may need a server-side proxy for a catalog that disallows it. The first search uses POST `/search`; pagination supports GET and POST next links.

The browser binding uses in-memory configuration. It does not include filesystem YAML loading, GeoParquet, background health monitors, asset signing, or capability-based catalog selection. Node.js and WASI are not validated targets.

For TypeScript, include `ESNext.Disposable` alongside your usual libraries because the generated bindings expose `Symbol.dispose`. Calling `client.free()` explicitly works too.
