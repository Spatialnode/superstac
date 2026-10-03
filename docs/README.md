# SuperSTAC documentation

A Fumadocs / Next.js site for **https://spatialnode.com/superstac**, with the approved logo, a custom landing page, search, light/dark themes, and Python/Rust/CLI guides. Source content lives in `content/docs`; branding originals stay in `assets` for the repository READMEs.

## Develop

Use Node **22.19+** (the latest Node 22 LTS patch is recommended).

```bash
cd docs
nvm use
npm install --global pnpm@9.8.0
pnpm install --frozen-lockfile
pnpm dev
```

Open **http://localhost:3000/superstac**. The reference pages start at `/superstac/docs`. The `/superstac` base path is intentionally enabled in development so broken asset and search URLs can be caught before deployment.

```bash
pnpm check
pnpm build
pnpm start
# In another terminal, while the production server is running:
pnpm test:links
```

`test:links` visits all authored docs routes, checks rendered internal links and assets, and exercises the search endpoint. Set `DOCS_TEST_ORIGIN` if the server is on another port.

## Authoring

- Add Markdown or MDX pages in `content/docs`.
- Set `title` and `description` in frontmatter and order pages in each folder's `meta.json`.
- Link between docs using `/docs/...`. Next.js adds `/superstac` for these links.
- Public asset/download URLs use `/superstac/...` explicitly; plain image URLs do not receive Next.js base-path handling.
- Edit the complete downloadable YAML in `public/examples/superstac.yml` and keep the quickstart snippet in sync.
- Build hooks copy the two approved SVGs from `assets` to `public/brand`. Do not edit generated copies.
- The Spatialnode footer badge is in `public/powered-by-spatialnode.svg`; its dark variant preserves the paths with white lettering and a dark background. Both follow the site's theme toggle.
- The runnable Python notebook lives in `public/notebooks/superstac-python-quickstart.ipynb`, served at `/superstac/notebooks/superstac-python-quickstart.ipynb`. Keep this single source of truth free of saved outputs, execution counts, credentials, and personal runtime metadata. Its documentation page is `content/docs/python/notebook.mdx`.
- Verify examples against the source when the alpha API changes. The docs record current implementation behavior, including limitations; they do not promise future features.

Commit `pnpm-lock.yaml` whenever dependencies change. The Docker build uses pnpm 9.8.0 with `--frozen-lockfile`; dependency drift fails the build.

## Python notebook

Download [the notebook](public/notebooks/superstac-python-quickstart.ipynb) and upload it to Colab, or open it locally with Jupyter. Its cells install the Python dependencies when the reader runs them, search two public catalogs, show a footprint map, and export a ZIP of the results. They have not been executed against live catalogs here.

The direct **Open in Colab** links target this file on GitHub's `main` branch and become usable after it is pushed there. Before publishing an updated notebook, run all cells in a fresh runtime, check the diagnostics and downloaded results, then clear outputs and execution counts before saving it back to the repository.

## GitHub deployment automation

[Deploy docs](../.github/workflows/docs-deploy.yml) runs on **published GitHub releases**, including prereleases, and through **Actions → Deploy docs → Run workflow**. Ordinary pushes and tag creation alone do not deploy. Publish a release only when its corresponding package is available; the workflow does not publish packages or wait for a separate package-publishing workflow.

The workflow checks out the release tag or the manually supplied Git ref. It builds an AMD64 image, runs the Dockerfile's type check and production build, starts the resulting container, and checks pages, links, assets, search, and the 404 response. Only then does it publish to GHCR and deploy through the Coolify API. Use an x86-64 Coolify server for this workflow; an ARM server needs a matching runner/image platform.

## Browser playground

The WASM guide is in `content/docs/wasm`. The playground is a small static app
in `public/playground`, embedded in the docs with `components/wasm-playground.tsx`.
It loads WASM only when a visitor searches. Keep API internals in the guide and
code example; use scene and catalog language in the results.

For local work, reuse an existing build without compiling Rust:

```sh
node docs/scripts/prepare-wasm.mjs
```

This copies `crates/wasm/pkg` into the ignored `docs/public/wasm` directory.
If it is missing, download the `wasm-web` artifact from the **WASM checks** workflow
and extract it there, or follow the binding's build instructions. Docs startup
never builds Rust. Open `/superstac/docs/wasm/playground` on the docs dev server.

The docs deployment workflow runs WASM checks for the same Git ref and downloads
its tested browser package before building the docs image. No generated WASM
binaries are committed. The playground tests run one headless browser against
fixture responses; live provider availability is not a CI requirement.
