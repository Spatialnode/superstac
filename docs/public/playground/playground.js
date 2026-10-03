const $ = (id) => document.getElementById(id);
const areas = {
  alps: { name: 'French Alps', bbox: [6, 45, 7, 46] },
  barcelona: { name: 'Barcelona coast', bbox: [1.9, 41.2, 2.4, 41.6] },
  nairobi: { name: 'Nairobi', bbox: [36.6, -1.5, 37, -1.1] },
};
const catalogs = {
  'earth-search': { name: 'Earth Search', url: 'https://earth-search.aws.element84.com/v1' },
  microsoft: { name: 'Planetary Computer', url: 'https://planetarycomputer.microsoft.com/api/stac/v1' },
};
let wasm;
let latest;
let busy = false;
function selection() {
  const ids = [...document.querySelectorAll('input[name=catalog]:checked')].map(input => input.value);
  const area = areas[$('area').value];
  return { ids, area, from: $('from').value, to: $('to').value,
    config: { catalogs: ids.map(id => ({ id, url: catalogs[id].url })),
      settings: { per_catalog_timeout_seconds: 20, max_retry_attempts: 1 } },
    query: { collections: ['sentinel-2-l2a'], bbox: area.bbox,
      datetime: `${$('from').value}T00:00:00Z/${$('to').value}T23:59:59Z`, limit: Number($('limit').value) } };
}
function updateCode() {
  const { config, query } = selection();
  $('code').textContent = `import init, { SuperSTAC } from './pkg/superstac_wasm.js';\n\nawait init();\nconst client = new SuperSTAC(${JSON.stringify(config, null, 2)});\ntry {\n  const result = await client.search(${JSON.stringify(query, null, 2).replaceAll('\n', '\n  ')});\n  console.log(result.items, result.metadata.failures);\n} finally {\n  client.free();\n}`;
  $('copy-status').textContent = '';
}
function el(tag, text, className) {
  const node = document.createElement(tag);
  if (text !== undefined) node.textContent = text;
  if (className) node.className = className;
  return node;
}
function safeUrl(value) {
  try { const url = new URL(value); return ['https:', 'http:'].includes(url.protocol) ? url.href : null; }
  catch { return null; }
}
function render(response, selected) {
  latest = response;
  const { metadata, items } = response;
  $('empty').hidden = true;
  $('result-content').hidden = false;
  $('result-title').textContent = `${items.length} ${items.length === 1 ? 'scene' : 'scenes'} found`;
  $('result-query').textContent = `${selected.area.name} · ${selected.from} to ${selected.to}`;
  $('download').disabled = items.length === 0;
  $('summary').replaceChildren(
    el('span', `${metadata.catalogs_succeeded} of ${metadata.catalogs_queried} catalogs responded`),
    el('span', `${metadata.duplicates_removed} duplicate ${metadata.duplicates_removed === 1 ? 'record' : 'records'} combined`),
    el('span', `Up to ${selected.query.limit} scenes per catalog`));
  $('catalog-status').replaceChildren();
  for (const id of selected.ids) {
    const failure = metadata.failures.find(entry => entry.catalog_id === id);
    const count = items.filter(entry => entry.seen_in.includes(id)).length;
    const note = el('div', failure ? `${catalogs[id].name} couldn’t be reached. You can try again or search the other catalog.`
      : `${catalogs[id].name} returned ${count} ${count === 1 ? 'scene' : 'scenes'}.`, `catalog-note${failure ? ' failure' : ''}`);
    if (failure) {
      const details = el('details'); details.append(el('summary', 'Error details'), el('pre', failure.reason)); note.append(details);
    }
    $('catalog-status').append(note);
  }
  $('scenes').replaceChildren();
  if (!items.length) {
    $('scenes').append(el('p', metadata.catalogs_succeeded ? 'No scenes matched this search. Try a wider date range or another area.'
      : 'Neither catalog returned results. Check your connection, then try again.', 'hint'));
  }
  for (const entry of items) {
    const { item } = entry;
    const card = el('article', undefined, 'scene');
    const captured = item.properties.datetime ?? item.properties.start_datetime;
    const date = new Date(captured);
    const title = captured && Number.isFinite(date.getTime()) ? new Intl.DateTimeFormat('en', { dateStyle: 'medium', timeZone: 'UTC' }).format(date) : 'Capture date unavailable';
    card.append(el('h3', title), el('div', item.id, 'scene-id'));
    const cloud = item.properties['eo:cloud_cover'];
    card.append(el('p', `${typeof cloud === 'number' && Number.isFinite(cloud) ? `${Math.round(cloud)}% cloud cover` : 'Cloud cover not reported'} · ${Object.keys(item.assets ?? {}).length} assets`));
    const sources = el('div', undefined, 'sources');
    for (const source of entry.seen_in) sources.append(el('span', catalogs[source]?.name ?? source, 'source'));
    card.append(sources);
    const href = safeUrl(item.links?.find(link => link.rel === 'self')?.href);
    if (href) { const link = el('a', 'View catalog record ↗'); link.href = href; link.target = '_blank'; link.rel = 'noopener noreferrer'; const p = el('p'); p.append(link); card.append(p); }
    $('scenes').append(card);
  }
  $('response').textContent = JSON.stringify(response, null, 2);
}
$('search-form').addEventListener('change', updateCode);
$('search-form').addEventListener('submit', async event => {
  event.preventDefault();
  if (busy) return;
  const selected = selection();
  const validation = !selected.ids.length ? 'Select at least one catalog.' : selected.from > selected.to ? 'The end date must be on or after the start date.' : '';
  $('validation').textContent = validation; $('validation').hidden = !validation;
  if (validation) return;
  busy = true;
  $('controls').disabled = true; $('results').setAttribute('aria-busy', 'true');
  $('search-button').textContent = 'Searching…';
  $('status').textContent = `Searching ${selected.ids.map(id => catalogs[id].name).join(' and ')}… This can take up to 20 seconds.`;
  let client;
  try {
    if (!wasm) {
      // Keep the generated module out of the docs bundle; load it on first search.
      const module = await import('../wasm/superstac_wasm.js');
      await module.default(); wasm = module;
    }
    client = new wasm.SuperSTAC(selected.config);
    const response = await client.search(selected.query);
    render(response, selected);
    $('status').textContent = response.metadata.catalogs_failed ? 'Search finished. Some catalogs could not return results; see the details below.' : 'Search finished.';
  } catch {
    $('status').textContent = 'The search could not start. Check your connection and try again. If this keeps happening, reload the page.';
  } finally {
    client?.free(); busy = false; $('controls').disabled = false;
    $('results').setAttribute('aria-busy', 'false'); $('search-button').textContent = 'Search scenes →';
  }
});
$('download').addEventListener('click', () => {
  if (!latest?.items.length) return;
  const blob = new Blob([JSON.stringify({ type: 'FeatureCollection', features: latest.items.map(entry => entry.item) }, null, 2)], { type: 'application/geo+json' });
  const url = URL.createObjectURL(blob); const link = el('a'); link.href = url; link.download = 'superstac-scenes.geojson';
  document.body.append(link); link.click(); link.remove(); setTimeout(() => URL.revokeObjectURL(url), 1000);
});
$('copy').addEventListener('click', async () => {
  try { await navigator.clipboard.writeText($('code').textContent); $('copy-status').textContent = 'Code copied.'; }
  catch { $('copy-status').textContent = 'Select the code below to copy it.'; }
});
updateCode();
// The docs embed follows the site's theme and grows with the result list.
window.addEventListener('message', event => {
  if (event.origin === location.origin && event.source === parent && event.data?.type === 'superstac-theme') {
    document.documentElement.dataset.theme = event.data.dark ? 'dark' : 'light';
  }
});
if (parent !== window) new ResizeObserver(() => parent.postMessage({ type: 'superstac-height', height: document.body.scrollHeight }, location.origin)).observe(document.body);
