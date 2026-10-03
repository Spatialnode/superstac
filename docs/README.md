# SuperSTAC documentation

Source for [spatialnode.com/superstac](https://spatialnode.com/superstac), built with
Fumadocs and Next.js.

## Run locally

Use Node 22.19 or newer and pnpm 9.8.0:

```bash
cd docs
nvm use
npm install --global pnpm@9.8.0
pnpm install --frozen-lockfile
pnpm dev
```

Open <http://localhost:3000/superstac>.

## Edit the docs

- Pages live in `content/docs/v0.3`. Set each page’s `title` and `description` at the top
  of the file, and add it to the folder’s `meta.json` to include it in navigation.
- Keep links within their version: `/docs/v0.3/...`. Versioned downloads live in
  `public/versions/v0.3`, served at `/superstac/versions/v0.3/...`.
- Keep the quickstart’s YAML example in sync with `public/versions/v0.3/examples/superstac.yml`.
- Versioned notebooks live in `public/versions/v0.3/notebooks`. Run changed cells before publishing, then
  clear saved outputs and execution counts.
- Edit logos in `assets`; the build copies them to `public/brand`.

For the next minor release, copy the current version’s content and downloads to
a new version folder, update its links and examples, and add it to `versions.json`
and `content/docs/meta.json`. Set `latest` in `versions.json` when it is ready. Keep
older folders for readers using those releases; corrections can still be made.

Commit `pnpm-lock.yaml` when changing dependencies.

Use the pnpm version pinned in `package.json` (9.8.0, also used in CI). Avoid mixing `npm install` and pnpm in this directory; mixed dependency folders can break CSS module resolution.

## Check changes

```bash
pnpm check
pnpm build
pnpm start
```

With the server running, open another terminal in `docs` and run `pnpm test:links`.
It checks pages, links, assets, and search. Set `DOCS_TEST_ORIGIN` if you use a
port other than 3000.

## Publish

The [Docs image workflow](../.github/workflows/docs-checks.yml) builds the browser
package, runs its checks, and puts the tested WASM files into the docs image.
It then checks the docs build, pages, links, and WASM asset MIME type. These builds
run in GitHub Actions, not on your laptop. Every build contains all docs versions.

Pull requests and manual runs build and test without publishing. Pushes to `main`
publish the passing image as `ghcr.io/<owner>/<repo>-docs:sha-<commit>` and update
`:production`. Package releases do not replace the docs with a release checkout.

Configure Coolify as a **Docker Image** application using that `:production` image,
port 3000, and the existing domain. The image serves the `/superstac` base path.
If the GHCR package is private, give Coolify registry credentials with read access.
Pull/redeploy after the workflow succeeds; to roll back, select a previous
`sha-<commit>` image. The workflow does not call a deployment webhook or change
Coolify settings. It produces an AMD64 image for an x86-64 host.

For a local Docker build, first run `node docs/scripts/prepare-wasm.mjs` from the
repository root. Docker uses `docs/` as its context and does not compile Rust.

## Browser playground

The guide is in `content/docs/v0.3/wasm`. The playground is a small static app in
`public/playground`, served at `/superstac/playground` through a Next.js rewrite.
The landing page links directly to it. Its modules separate configuration, catalog
editing, activity recording, map controls, viewer links, and results so features can grow independently. It loads WASM
on the first search or collection lookup. Its theme follows the system by default
and can be changed in the page header. Keep scene and catalog
language in the results; API details belong in the guide and code example.

Reuse an existing package locally without compiling Rust:

```sh
node docs/scripts/prepare-wasm.mjs
```

This copies `crates/wasm/pkg` into the ignored `docs/public/wasm` directory. If it
is missing, download the `wasm-web` artifact from **WASM checks** and extract it
there, or follow the binding's build instructions. Docs startup never builds
Rust. Open `/superstac/playground` on the docs dev server.

Playground tests use the actual WASM package with fixture catalog responses in
one headless browser. Run `npm run test:playground` in `crates/wasm`. Provider
availability is not a CI requirement, and no generated binaries are committed.

The overview map uses pinned MapLibre GL JS 6.11.2 from unpkg and OpenFreeMap
basemaps (Positron and Dark), preserving their OpenStreetMap/OpenMapTiles attribution. One map worker is used. Its layout
follows shadcn/mapcn patterns, while the static app keeps the docs site's palette
and logo. It does not install the React mapcn component. Map loading is independent
of WASM: searches and external viewer links still work if WebGL or the CDN fails.
Scene previews stay on OpenFreeMap. JPEG/PNG/WebP image assets are placed over
scene bounds with an explicit approximate-alignment label. GeoTIFFs use the lazy,
pinned `@geomatico/maplibre-cog-protocol@0.5.0` reader only when selected. Asset
access still depends on CORS, HTTP ranges and provider authentication. STAC Map
and STAC Browser are external links only, so their basemaps do not load in our page.

`modules/activity.js` observes catalog Fetch calls during playground operations;
it passes the original arguments and Response through, without adding headers.
Timing ends at response headers. The monitor is page-local, bounded to 200 events,
and omits query values and bodies. The standalone WASM API is unchanged. Health is
last-observed status, with manual connection checks and no background polling.

The playground’s **Browse STAC Index** picker loads `https://stacindex.org/api/catalogs` on demand. `modules/stac-index.js` filters public API entries, removes duplicate endpoints, and fills the catalog editor for review; it does not probe the directory’s endpoints. `modules/alias-editor.js` supplies editable collection and asset name pairs. Scene results overlay the map and collapse when previewing an asset.
