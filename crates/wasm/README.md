# SuperSTAC WebAssembly

Run live federated STAC searches in a browser using the shared Rust search logic.
This package supports catalog configuration, collection discovery, pagination,
collection/asset aliases, deduplication, provenance, concurrency limits, retries,
per-attempt timeouts, and per-catalog failures.

## Build and try

From the repository root, using Rust 1.88 or newer:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.13.1 --locked
wasm-pack build crates/wasm --target web --release --locked
python3 -m http.server 8000 --directory crates/wasm
```

Open http://localhost:8000/examples/ to try a live Earth Search query.
`crates/wasm/pkg` contains the generated JavaScript module, WebAssembly binary,
TypeScript declarations, and package metadata. Build artifacts are ignored by Git.
The package has not been published to npm.

## JavaScript / TypeScript

```ts
import init, { SuperSTAC } from './pkg/superstac_wasm.js';

await init();
const client = new SuperSTAC({
  catalogs: [{
    id: 'earth-search',
    url: 'https://earth-search.aws.element84.com/v1',
    // Optional maps: canonical name -> provider-local name.
    collection_aliases: {},
    asset_aliases: {},
  }],
  settings: {
    max_concurrent_catalogs: 8,
    per_catalog_timeout_seconds: 30,
    max_retry_attempts: 2, // Total attempts, including the first request.
    max_items_per_catalog: 1000,
    deduplicate_items: true,
    unify_response: true,
  },
});

const result = await client.search({
  collections: ['sentinel-2-l2a'],
  bbox: [6, 49, 7, 50],
  limit: 20,
});
console.log(result.items, result.metadata.failures);
console.log(await client.listCollections());
client.free(); // Once all outstanding calls have completed.
```

`search()` returns a Promise containing `{ items, metadata }`. Each result entry
has `item` (the STAC Feature), `catalog_id`, and `seen_in` provenance. It is not
itself a GeoJSON FeatureCollection. Missing `collections` means no collection
filter. `limit` defaults to 10 and applies **per catalog**, capped by
`max_items_per_catalog`; deduplication then merges the results.

Invalid configuration throws a JavaScript Error; invalid queries reject the
Promise. Individual catalog failures, including timeouts and CORS failures,
are recorded in `metadata.failures`. Even when every catalog fails, the Promise
resolves with an empty result and failure metadata: always inspect it.

`listCollections()` returns one entry per catalog with `collections` and a
nullable `error`. Each collection includes its `canonical_id` and original
`collection` metadata. Discovery follows next links, uses the configured
concurrency/timeout, and limits results to 10,000 collections per catalog.
It does not retry or change which catalogs subsequent searches query.

## Browser scope

- Endpoints must allow browser requests through CORS. A server-side proxy may
  be needed for catalogs that do not. WASM cannot bypass browser restrictions.
- The first search request uses STAC POST `/search`; pagination supports GET
  and POST next links, relative URLs, headers, and merged POST bodies.
- Configuration stays in memory. No filesystem YAML loading, GeoParquet,
  background health monitoring, or engine capability-based catalog filtering.
- This build targets browsers. Node.js and WASI are not validated targets.
- Browser timers can be delayed in background tabs. Timeouts cover each full
  paginated search attempt, and failed attempts discard their partial items.
- Native search uses `stac` 0.16 and `stac-io`; WASM uses `stac` 0.17.0 without
  filesystem features and a Fetch transport. The exact WASM version is pinned
  because newer STAC releases conflict with the native GeoParquet dependency graph. WASM backend futures are local;
  native backend futures retain their Send bounds.

## Tests

```sh
cd crates/wasm
npm ci
npm run build
npx playwright install chromium
npm test
npx tsc --noEmit --strict --target ES2022 --module ESNext --moduleResolution bundler tests/types.ts
```

Set `CHROME_PATH` to use an existing Chrome executable. The browser suite runs
against a local STAC fixture server and checks pagination, retries, timeouts,
aliases, deduplication, result serialization, validation, and partial failures.
Set `SUPERSTAC_LIVE_TEST=1 npm test` to also query Earth Search; this optional
check needs internet access and the provider's CORS support.
