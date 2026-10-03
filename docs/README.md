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

Coolify builds and deploys this site when changes are pushed to `main`. Each build
includes all versions in `content/docs`; publishing a package release does not
replace the docs with that release’s checkout.

The [Docs checks workflow](../.github/workflows/docs-checks.yml) builds and tests
the site on pull requests. It can also be run manually and does not deploy.
