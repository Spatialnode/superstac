import { readdir, readFile } from 'node:fs/promises';

const origin = process.env.DOCS_TEST_ORIGIN ?? 'http://127.0.0.1:3000';
const base = '/superstac';
const content = new URL('../content/docs/', import.meta.url);
const files = await readdir(content, { recursive: true });
const routes = [base, ...files.filter((f) => /\.mdx?$/.test(f)).map((f) => {
  const slug = f.replace(/\.mdx?$/, '').replace(/(^|\/)index$/, '');
  return `${base}/docs${slug ? `/${slug}` : ''}`;
})];
const failures = [];
const resources = new Set([`${base}/favicon.svg`, `${base}/brand/superstac-logo.svg`, `${base}/brand/superstac-logo-dark.svg`, `${base}/examples/superstac.yml`, `${base}/sitemap.xml`]);
const documents = new Map();
const anchors = [];
for (const route of routes) {
  const res = await fetch(origin + route);
  if (!res.ok) { failures.push(`${route}: ${res.status}`); continue; }
  const html = await res.text();
  documents.set(route.replace(/\/$/, ''), html);
  for (const match of html.matchAll(/(?:href|src)="([^"<>]+)"/g)) {
    const value = match[1].replaceAll('&amp;', '&');
    if (value.startsWith('data:') || value.startsWith('mailto:')) continue;
    const url = new URL(value, origin + route);
    if (url.origin !== new URL(origin).origin) continue;
    if (!url.pathname.startsWith(`${base}/`) && url.pathname !== base) {
      failures.push(`${route}: URL escaped basePath: ${value}`); continue;
    }
    resources.add(url.pathname + url.search);
    if (url.hash) anchors.push([route, url.pathname.replace(/\/$/, ''), decodeURIComponent(url.hash.slice(1))]);
  }
  if (!html.includes(`https://spatialnode.com${route}`)) failures.push(`${route}: missing production canonical`);
}
for (const resource of resources) {
  const res = await fetch(origin + resource);
  if (!res.ok) failures.push(`${resource}: ${res.status}`);
}
for (const [from, route, id] of anchors) {
  const html = documents.get(route);
  if (html && !html.includes(`id="${id}"`)) failures.push(`${from}: missing anchor ${route}#${id}`);
}
const search = await fetch(`${origin}${base}/api/search?query=deduplication`);
if (!search.ok) failures.push(`search: ${search.status}`);
else {
  const results = await search.json();
  if (!Array.isArray(results) || results.length === 0) failures.push('Search returned no results for deduplication');
}
const notFound = await fetch(`${origin}${base}/docs/not-a-real-page`);
if (notFound.status !== 404) failures.push(`Missing page returned ${notFound.status}, expected 404`);
const sample = await readFile(new URL('../public/examples/superstac.yml', import.meta.url), 'utf8');
const quickstart = await readFile(new URL('../content/docs/start/quickstart.mdx', import.meta.url), 'utf8');
if (!quickstart.includes(sample.trim())) failures.push('Quickstart YAML drifted from downloadable sample');
if (failures.length) { console.error(failures.join('\n')); process.exit(1); }
console.log(`Verified ${routes.length} pages, ${resources.size} linked routes/assets, anchors, canonical URLs, search, 404, and sample config consistency under ${base}.`);
