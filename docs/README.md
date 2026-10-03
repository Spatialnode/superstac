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

- Pages live in `content/docs`. Set each page’s `title` and `description` at the top
  of the file, and add it to the folder’s `meta.json` to include it in navigation.
- Link to other docs with `/docs/...`. Use `/superstac/...` for images and downloads
  in `public`.
- Keep the quickstart’s YAML example in sync with `public/examples/superstac.yml`.
- Notebooks live in `public/notebooks`. Run changed cells before publishing, then
  clear saved outputs and execution counts.
- Edit logos in `assets`; the build copies them to `public/brand`.

Commit `pnpm-lock.yaml` when changing dependencies.

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

The [Deploy docs workflow](../.github/workflows/docs-deploy.yml) runs when a GitHub
release is published, including prereleases. You can also run it from **Actions →
Deploy docs → Run workflow**. Pushes alone do not deploy the site.

The workflow checks and builds the site, tests the container, and deploys it to
Coolify. It uses an x86-64 image. Publish the corresponding packages before
publishing a release; this workflow only deploys the docs.
