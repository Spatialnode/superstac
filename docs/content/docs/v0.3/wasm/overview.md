---
title: Use SuperSTAC in the browser
description: Search STAC catalogs from JavaScript or TypeScript with the WebAssembly binding.
---

The browser binding runs SuperSTAC's Rust search logic in your JavaScript app. Register catalogs, send one query, and receive combined results with source information.

[Try the playground](/docs/v0.3/wasm/playground) to see it without installing anything.

## Install

You do not need Rust to use SuperSTAC in a browser app. The browser package includes the compiled WebAssembly module and TypeScript declarations.

The planned npm package name is `@spatialnode/superstac-wasm`. Once released, installation will be:

```bash
npm install @spatialnode/superstac-wasm
```

For now, use a prebuilt package from a successful [WASM checks workflow run](https://github.com/spatialnode/superstac/actions/workflows/wasm.yml):

1. Open a successful run and download the **wasm-web** artifact. GitHub requires sign-in to download workflow artifacts; they are available until they expire.
2. Extract it into `vendor/superstac-wasm` in your app. The folder should contain `package.json`, `superstac_wasm.js`, `superstac_wasm_bg.wasm`, and `superstac_wasm.d.ts`.
3. Install it from your app’s root:

```bash
npm install ./vendor/superstac-wasm
```

This installs the same `@spatialnode/superstac-wasm` import used below, without compiling anything. Use an artifact built with the `spatialnode` npm scope; older artifacts may have an unscoped package name or lack `package.json`. If no artifact is available, [try the playground](/docs/v0.3/wasm/playground) or see the optional [source build guide](/docs/v0.3/wasm/development).

## Search two catalogs

Put this in your app’s browser code. For frameworks with server rendering, initialize the client on the browser side. Your bundler must serve the package’s `.wasm` file alongside its JavaScript module.

```js
import init, { SuperSTAC } from '@spatialnode/superstac-wasm';

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

Catalogs can also include `collection_aliases` and `asset_aliases`, using the [same canonical-to-provider mappings](/docs/v0.3/guides/aliases) as the other interfaces.

## Request identification and logs

The browser binding uses the browser’s User-Agent. It does not append a SuperSTAC
identifier or install a Rust tracing subscriber. Catalog errors are available in
`metadata.failures`; complete browser requests can be inspected in Developer Tools.
The [playground](/docs/v0.3/wasm/playground) adds its own activity panel with request
response times, HTTP outcomes, and total operation time, without changing request
headers. This instrumentation belongs to the playground, not the npm binding API.

## Browser requirements

Catalogs must allow browser requests through CORS. WASM cannot bypass that restriction; an application may need a server-side proxy for a catalog that disallows it. The first search uses POST `/search`; pagination supports GET and POST next links.

The browser binding uses in-memory configuration. It does not include filesystem YAML loading, GeoParquet, background health monitors, asset signing, or capability-based catalog selection. Node.js and WASI are not validated targets.

For TypeScript, include `ESNext.Disposable` alongside your usual libraries because the generated bindings expose `Symbol.dispose`. Calling `client.free()` explicitly works too.
