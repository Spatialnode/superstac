import { $, el, button } from './dom.js';
import { catalogUrl } from './model.js';

export const INDEX_URL = 'https://stacindex.org/api/catalogs';
export const urlKey = value => { try { return catalogUrl(value).replace(/\/+$/, ''); } catch { return ''; } };

export function publicApis(data) {
  if (!Array.isArray(data)) throw new Error('Unexpected catalog directory response.');
  const seen = new Set();
  return data.flatMap(entry => {
    if (!entry || entry.isApi !== true || entry.isPrivate === true || entry.access !== 'public') return [];
    const url = typeof entry.url === 'string' ? urlKey(entry.url) : '';
    if (!url || /\.json$/i.test(new URL(url).pathname) || seen.has(url)) return [];
    seen.add(url);
    const name = typeof entry.title === 'string' && entry.title.trim() ? entry.title.trim() : new URL(url).hostname;
    const summary = typeof entry.summary === 'string' ? entry.summary.replace(/\[([^\]]+)\]\([^)]*\)/g, '$1').replace(/\s+/g, ' ').trim().slice(0, 360) : '';
    return [{ name, url, summary, slug: typeof entry.slug === 'string' ? entry.slug : name }];
  }).sort((a, b) => a.name.localeCompare(b.name));
}

export function directoryCatalog(entry, existing) {
  const base = entry.slug.toLowerCase().replace(/[^a-z0-9_-]+/g, '-').replace(/^-|-$/g, '') || 'catalog';
  let id = base, suffix = 2;
  while (existing.some(c => c.id === id)) id = `${base}-${suffix++}`;
  return { id, name: entry.name, url: entry.url, provider: '', collection_aliases: {}, asset_aliases: {} };
}

export function catalogDirectory(getCatalogs, add, isBusy) {
  let entries, controller, limit = 20;
  function render() {
    if (!entries) return;
    const query = $('index-search').value.trim().toLowerCase();
    const matches = entries.filter(e => `${e.name} ${e.summary} ${e.url}`.toLowerCase().includes(query));
    const existing = new Set(getCatalogs().map(c => urlKey(c.url)));
    $('index-list').replaceChildren();
    for (const entry of matches.slice(0, limit)) {
      const card = el('article', undefined, 'index-entry');
      card.append(el('h3', entry.name), el('p', entry.summary, 'hint'), el('p', entry.url, 'catalog-url'));
      const added = existing.has(entry.url);
      const choose = button(added ? 'Already added' : 'Choose catalog', () => {
        if (isBusy()) return;
        $('index-dialog').close(); add(directoryCatalog(entry, getCatalogs()));
      });
      choose.disabled = added || isBusy(); card.append(choose); $('index-list').append(card);
    }
    $('index-status').textContent = matches.length ? `${Math.min(limit, matches.length)} of ${matches.length} catalogs` : 'No catalogs match. Try another name or topic.';
    $('index-more').hidden = matches.length <= limit;
  }
  async function load() {
    controller?.abort(); const request = new AbortController(); controller = request;
    const timer = setTimeout(() => request.abort(), 12000);
    $('index-status').textContent = 'Loading catalogs…'; $('index-retry').hidden = true;
    $('index-list').replaceChildren(); $('index-more').hidden = true;
    try {
      const response = await fetch(INDEX_URL, { signal: request.signal, credentials: 'omit' });
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      const data = await response.json();
      if (request.signal.aborted) return;
      entries = publicApis(data); render();
    } catch {
      if (controller === request && $('index-dialog').open) {
        $('index-status').textContent = 'STAC Index could not load. Try again, or close this window and add a catalog by URL.';
        $('index-retry').hidden = false;
      }
    } finally { clearTimeout(timer); }
  }
  $('browse-index').addEventListener('click', () => {
    if (isBusy()) return;
    $('index-dialog').showModal(); $('index-search').focus();
    if (entries) render(); else load();
  });
  $('index-dialog').addEventListener('close', () => controller?.abort());
  $('index-retry').addEventListener('click', load);
  $('index-search').addEventListener('input', () => { limit = 20; render(); });
  $('index-more').addEventListener('click', () => { limit += 20; render(); });
}
