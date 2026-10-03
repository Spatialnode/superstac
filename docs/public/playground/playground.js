import { $, el, button, errorMessage } from './modules/dom.js';
import { defaults, DEFAULT_SETTINGS, AREAS, configFor, queryFor, example } from './modules/model.js';
import { catalogDirectory } from './modules/stac-index.js';
import { catalogEditor } from './modules/catalogs.js';
import { resultView } from './modules/results.js';
import { activityMonitor } from './modules/activity.js';
import { overviewMap } from './modules/map.js';

let state = defaults(), busy = false, wasm;
const activity = activityMonitor(() => catalogs.render());
const overview = overviewMap(bbox => {
  if (busy) return;
  $('area').value = 'custom'; $('geometry').value = '';
  ['west', 'south', 'east', 'north'].forEach((id, i) => { $(id).value = bbox[i]; });
  tab('search'); updateCode();
}, index => results.preview(index));
const results = resultView(overview);
const catalogs = catalogEditor(() => state, changed, () => busy, activity.badge);
catalogDirectory(() => state.catalogs, catalogs.addFromIndex, () => busy);
function values() {
  return { ...Object.fromEntries(['collections', 'area', 'from', 'to', 'limit', 'ids', 'sortby', 'geometry'].map(id => [id, $(id).value])), bbox: ['west', 'south', 'east', 'north'].map(id => $(id).value) };
}
function updateCode() {
  try { const query = queryFor(values()); overview.query(query); $('code').textContent = example(configFor(state), query); $('copy').disabled = false; }
  catch (error) { $('code').textContent = errorMessage(error); $('copy').disabled = true; }
  $('copy-status').textContent = '';
  $('bbox-fields').hidden = $('area').value !== 'custom';
  const selected = state.catalogs.filter(c => c.enabled);
  $('selected-catalogs').textContent = selected.length ? `Searching ${selected.map(c => c.name).join(' and ')}.` : 'No catalogs selected. Choose one in Catalogs.';
}
function changed() { catalogs.render(); updateCode(); $('discovery').replaceChildren(); activity.log('Workspace updated', `${state.catalogs.length} catalogs · ${state.providers.length} providers`, { ms: 0 }); }
function tab(name, focus = false) {
  controlsPanel(true);
  document.querySelector('.controls-body').scrollTop = 0;
  $('search-actions').hidden = name !== 'search';
  $('settings-actions').hidden = name !== 'settings';
  document.querySelectorAll('[data-tab]').forEach(b => {
    const active = b.dataset.tab === name; b.setAttribute('aria-selected', String(active)); b.tabIndex = active ? 0 : -1;
    $(`panel-${b.dataset.tab}`).hidden = !active; if (active && focus) b.focus();
  });
}
$('browse-collections').addEventListener('click', () => {
  if (busy) return;
  tab('catalogs', true); $('discover').click();
});
document.addEventListener('playground:edit-search', () => tab('search', true));
$('manage-catalogs').addEventListener('click', () => tab('catalogs', true));
function controlsPanel(open, focus = false) {
  $('workspace-controls').hidden = !open;
  document.body.dataset.controls = open ? 'open' : 'closed';
  $('toggle-controls').setAttribute('aria-expanded', String(open));
  if (open && matchMedia('(max-width: 900px)').matches) results.collapse();
  if (focus) (open ? document.querySelector('[data-tab][aria-selected="true"]') : $('toggle-controls')).focus({ preventScroll: true });
}
$('toggle-controls').addEventListener('click', () => {
  if ($('workspace-controls').hidden) tab('search', true);
  else controlsPanel(false, true);
});
$('close-controls').addEventListener('click', () => controlsPanel(false, true));
$('workspace-controls').addEventListener('keydown', event => {
  if (event.key === 'Escape') { event.stopPropagation(); controlsPanel(false, true); }
});
document.addEventListener('playground:preview', () => controlsPanel(false));
document.addEventListener('playground:results-open', () => {
  if (matchMedia('(max-width: 900px)').matches) controlsPanel(false);
});
const tabs = [...document.querySelectorAll('[data-tab]')];
tabs.forEach((b, i) => {
  b.addEventListener('click', () => tab(b.dataset.tab));
  b.addEventListener('keydown', event => {
    const next = { ArrowRight: (i + 1) % tabs.length, ArrowLeft: (i + tabs.length - 1) % tabs.length, Home: 0, End: tabs.length - 1 }[event.key];
    if (next !== undefined) { event.preventDefault(); tab(tabs[next].dataset.tab, true); }
  });
});
function lock(value) {
  busy = value;
  for (const id of ['search-button', 'apply-settings', 'reset-settings', 'browse-collections']) $(id).disabled = value;
  $('query-controls').disabled = value; $('settings-controls').disabled = value; $('reset').disabled = value;
  $('try-aerial').disabled = value; $('try-aliases').disabled = value; $('results').setAttribute('aria-busy', String(value)); catalogs.render();
}
async function clientFor(config) {
  if (!wasm) {
    const module = await import('/superstac/wasm/superstac_wasm.js'); await module.default(); wasm = module;
  }
  return new wasm.SuperSTAC(config);
}
$('search-form').addEventListener('input', updateCode);
$('search-form').addEventListener('submit', async event => {
  event.preventDefault(); if (busy) return;
  let config, query;
  try { config = configFor(state); query = queryFor(values()); $('validation').hidden = true; }
  catch (error) { $('validation').textContent = errorMessage(error); $('validation').hidden = false; return; }
  const snapshot = { unify: state.settings.unify_response, catalogs: structuredClone(state.catalogs), query, areaName: query.intersects ? 'Custom geometry' : AREAS[$('area').value]?.name ?? ($('area').value === 'custom' ? 'Custom area' : 'Anywhere') };
  activity.begin('Search', snapshot.catalogs.filter(c => c.enabled));
  controlsPanel(false); $('toggle-controls').focus({ preventScroll: true });
  lock(true); $('search-button').textContent = 'Searching…'; $('status').textContent = 'Searching your selected catalogs…';
  let client;
  try {
    client = await clientFor(config); const response = await client.search(query); results.render(response, snapshot); activity.outcome(response, snapshot.catalogs.filter(c => c.enabled));
    activity.finish(`${response.items.length} scenes · ${response.metadata.catalogs_failed} catalogs failed`, response.metadata.catalogs_failed > 0);
    $('status').textContent = response.metadata.catalogs_succeeded === 0 ? 'No catalogs could complete this search. See the details below.' : response.metadata.catalogs_failed ? 'Search finished. Some catalogs could not return results; see the details below.' : 'Search finished.';
  } catch (error) { activity.finish('Search could not complete. See the status message.', true); $('status').textContent = `The search could not finish. ${errorMessage(error)}`; }
  finally { client?.free(); lock(false); $('search-button').textContent = 'Search scenes →'; }
});
$('discover').addEventListener('click', async () => {
  if (busy) return; let client, config;
  try { config = configFor(state); }
  catch (error) { $('discovery').textContent = errorMessage(error); return; }
  activity.begin('Collection discovery', state.catalogs.filter(c => c.enabled));
  lock(true); $('discovery').textContent = 'Finding collections…';
  try {
    client = await clientFor(config); const response = await client.listCollections();
    $('discovery').replaceChildren(); $('discovery').scrollIntoView({ block: 'nearest' }); const suggestions = new Set();
    for (const result of response) {
      const group = el('details', undefined, 'collection-group'); group.open = response.length === 1;
      group.append(el('summary', `${state.catalogs.find(c => c.id === result.catalog_id)?.name ?? result.catalog_id} · ${result.collections.length} collections`));
      if (result.error) group.append(el('p', result.error, 'error'));
      const list = el('div', undefined, 'collection-list');
      for (const collection of result.collections) {
        suggestions.add(collection.canonical_id);
        const choose = button(collection.collection.title ?? collection.canonical_id, () => {
          $('collections').value = collection.canonical_id; tab('search', true); updateCode();
        }, 'collection-button');
        choose.append(el('span', collection.canonical_id, 'hint')); list.append(choose);
      }
      group.append(list); $('discovery').append(group);
    }
    if (!$('discovery').children.length) $('discovery').textContent = 'No collections were found. Check the selected catalogs or add another catalog.';
    $('discovery').scrollIntoView({ block: 'nearest' });
    activity.finish(`${response.reduce((sum, r) => sum + r.collections.length, 0)} collections found`, response.some(r => r.error));
    $('collection-suggestions').replaceChildren(...[...suggestions].map(id => new Option(id, id)));
  } catch (error) { activity.finish('Collection discovery failed', true); $('discovery').textContent = `Collections could not load. ${errorMessage(error)}`; }
  finally { client?.free(); lock(false); }
});
$('try-aerial').addEventListener('click', () => {
  if (busy) return;
  const url = 'https://api.imagery.hotosm.org/stac';
  state.catalogs.forEach(c => { c.enabled = false; });
  let catalog = state.catalogs.find(c => c.url.replace(/\/$/, '') === url);
  if (!catalog) {
    let id = 'hot-openaerialmap';
    while (state.catalogs.some(c => c.id === id)) id += '-example';
    catalog = { id, name: 'HOT OpenAerialMap', url, provider: '', collection_aliases: {}, asset_aliases: {} };
    state.catalogs.push(catalog);
  }
  catalog.enabled = true;
  $('search-form').reset(); $('collections').value = ''; $('area').value = 'anywhere';
  $('from').value = ''; $('to').value = ''; $('limit').value = '1';
  $('ids').value = '65cdcbd8-34f8-42d3-a758-1ebb69d3b149';
  changed(); tab('search', true);
  $('status').textContent = 'Aerial example ready. Search, then choose Preview on map to explore the GeoTIFF.';
});
$('try-aliases').addEventListener('click', () => {
  if (busy) return;
  state.catalogs.forEach(c => { c.enabled = false; });
  for (const sample of defaults().catalogs) {
    let catalog = state.catalogs.find(c => c.url === sample.url);
    if (!catalog) {
      while (state.catalogs.some(c => c.id === sample.id)) sample.id += '-example';
      catalog = sample; state.catalogs.push(catalog);
    }
    catalog.enabled = true; catalog.collection_aliases.optical = 'sentinel-2-l2a';
    catalog.asset_aliases.optical = { ...catalog.asset_aliases.optical, rgb: 'visual' };
  }
  state.settings.unify_response = true; showSettings(); $('collections').value = 'optical';
  changed(); tab('search', true);
  $('status').textContent = 'Alias example ready. Search for optical; returned images will use the name rgb.';
});
$('check-health').addEventListener('click', async () => {
  if (busy) return;
  const selected = state.catalogs.filter(c => c.enabled);
  if (!selected.length) { $('discovery').textContent = 'Select a catalog to check its connection.'; return; }
  lock(true); activity.begin('Connection check', selected); $('discovery').textContent = 'Checking selected catalogs…';
  try { await activity.check(selected); $('discovery').textContent = 'Connection checks finished. Each catalog shows its last response.'; }
  finally { activity.finish('See each catalog’s last response'); lock(false); }
});
function activityPanel(open) {
  $('activity-panel').hidden = !open; $('open-activity').setAttribute('aria-expanded', String(open));
  if (open) $('close-activity').focus(); else $('open-activity').focus();
}
$('open-activity').addEventListener('click', () => activityPanel($('activity-panel').hidden));
$('close-activity').addEventListener('click', () => activityPanel(false));
$('activity-panel').addEventListener('keydown', event => { if (event.key === 'Escape') activityPanel(false); });
$('settings-form').addEventListener('input', () => {
  $('settings-status').textContent = 'Unsaved changes. Apply settings to use them in searches and code.';
  $('apply-settings').textContent = 'Apply changes';
});
function showSettings() {
  $('apply-settings').textContent = 'Apply settings';
  for (const [key, value] of Object.entries(state.settings)) { if (typeof value === 'boolean') $(key).checked = value; else $(key).value = value; }
}
$('settings-form').addEventListener('submit', event => {
  event.preventDefault(); if (busy) return;
  state.settings = Object.fromEntries(Object.entries(DEFAULT_SETTINGS).map(([key, value]) => [key, typeof value === 'boolean' ? $(key).checked : Number($(key).value)]));
  $('apply-settings').textContent = 'Apply settings';
  updateCode(); activity.log('Settings applied', 'The next search will use these settings.', { ms: 0 }); $('settings-status').textContent = 'Settings applied to your next search.';
});
$('reset-settings').addEventListener('click', () => {
  if (busy) return; state.settings = { ...DEFAULT_SETTINGS }; showSettings(); updateCode(); $('settings-status').textContent = 'Default settings restored.';
});
$('reset').addEventListener('click', () => {
  if (busy) return; state = defaults(); $('search-form').reset(); showSettings(); changed(); results.clear();
  $('validation').hidden = true; $('settings-status').textContent = ''; $('status').textContent = 'Defaults restored. Ready to search.'; tab('search');
});
$('open-code').addEventListener('click', () => {
  updateCode(); $('code-dialog').showModal(); $('copy').focus();
});
$('copy').addEventListener('click', async () => {
  try { await navigator.clipboard.writeText($('code').textContent); $('copy-status').textContent = 'Code copied.'; }
  catch {
    const range = document.createRange(); range.selectNodeContents($('code'));
    const selection = window.getSelection(); selection.removeAllRanges(); selection.addRange(range);
    $('code').focus(); $('copy-status').textContent = 'Code selected. Press Ctrl+C or ⌘C to copy.';
  }
});
let theme;
try { theme = localStorage.getItem('superstac-playground-theme'); } catch { /* Storage may be unavailable. */ }
document.documentElement.dataset.theme = theme ?? (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
$('theme').addEventListener('click', () => {
  const next = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark'; document.documentElement.dataset.theme = next;
  overview.theme();
  try { localStorage.setItem('superstac-playground-theme', next); } catch { /* The theme still applies to this page. */ }
});
changed();
overview.start();
