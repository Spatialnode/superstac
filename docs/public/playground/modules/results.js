import { $, el, button, link } from './dom.js';
import { httpUrl } from './model.js';
import { itemUrl, thumbnailCandidates, viewerLinks, previewAssets } from './viewers.js';

export function resultView(overview) {
  let response, selected, page = 0;
  const pageSize = 12;
  let previewEntry, assets = [];
  function closeMap() {
    previewEntry = undefined; overview.clearPreview();
    $('close-map').hidden = true; $('viewer-links').hidden = true; $('preview-controls').hidden = true;
    $('viewer-title').textContent = 'Search area'; overview.resize();
    $('viewer-caption').textContent = 'Move the map to explore. Search to see scene footprints and imagery.';
  }
  function preview(entry, href) {
    if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
    document.dispatchEvent(new Event('playground:preview'));
    previewEntry = entry; $('close-map').hidden = false; collapseResults(true);
    $('viewer-title').textContent = entry.item.id;
    $('viewer-caption').textContent = 'Scene and asset preview · OpenFreeMap';
    const links = viewerLinks(href);
    $('viewer-links').replaceChildren(...(links ? [link('Open in STAC Map ↗', links.map), link('Open in STAC Browser ↗', links.browser)] : []));
    $('viewer-links').hidden = !links;
    const base = href ?? selected.catalogs.find(c => c.id === entry.catalog_id)?.url + '/';
    const catalog = selected.catalogs.find(c => c.id === entry.catalog_id);
    assets = previewAssets(entry.item, base, selected.unify ? catalog?.asset_aliases?.[entry.item.collection] : {});
    $('preview-asset').replaceChildren(new Option('Footprint only', ''), ...assets.map(a => new Option(`${a.title}${a.cog ? ' · GeoTIFF' : ' · image'}`, a.id)));
    const first = assets.find(a => a.tilejson && a.id === 'visual') ?? assets.find(a => a.tilejson) ?? assets.find(a => a.thumbnail) ?? assets.find(a => !a.cog);
    $('preview-asset').value = first?.id ?? '';
    $('preview-controls').hidden = false; overview.preview(entry, first);
    $('viewer').scrollIntoView({ behavior: 'instant', block: 'nearest' });
    $('close-map').focus({ preventScroll: true });
  }
  $('preview-asset').addEventListener('change', () => {
    if (previewEntry) overview.preview(previewEntry, assets.find(a => a.id === $('preview-asset').value));
  });
  function cards() {
    $('scenes').replaceChildren();
    for (const entry of response.items.slice(page * pageSize, (page + 1) * pageSize)) {
      const { item } = entry; const href = itemUrl(entry, selected.catalogs);
      const base = href ?? selected.catalogs.find(c => c.id === entry.catalog_id)?.url + '/';
      const card = el('article', undefined, 'scene');
      const picture = el('div', undefined, 'scene-picture');
      const previews = thumbnailCandidates(item, base);
      let nextPreview = 0;
      const fallback = () => picture.replaceChildren(el('span', 'No preview available', 'hint'));
      if (previews.length) {
        const img = el('img'); img.alt = `Preview of ${item.id}`; img.loading = 'lazy'; img.referrerPolicy = 'no-referrer';
        img.addEventListener('error', () => {
          if (nextPreview < previews.length) img.src = previews[nextPreview++];
          else fallback();
        });
        img.src = previews[nextPreview++]; picture.append(img);
      } else fallback();
      card.append(picture);
      const body = el('div', undefined, 'scene-body');
      const captured = item.properties?.datetime ?? item.properties?.start_datetime; const date = new Date(captured);
      body.append(el('h3', captured && Number.isFinite(date.getTime()) ? new Intl.DateTimeFormat('en', { dateStyle: 'medium', timeZone: 'UTC' }).format(date) : 'Capture date unavailable'), el('p', item.id, 'scene-id'));
      const cloud = item.properties?.['eo:cloud_cover']; const assets = Object.entries(item.assets ?? {});
      body.append(el('p', `${typeof cloud === 'number' && Number.isFinite(cloud) ? `${Math.round(cloud)}% cloud cover` : 'Cloud cover not reported'} · ${assets.length} assets`, 'hint'));
      const sourceCatalog = selected.catalogs.find(c => c.id === entry.catalog_id);
      if (selected.unify && sourceCatalog) {
        const localCollection = sourceCatalog.collection_aliases?.[item.collection];
        const mappings = Object.entries(sourceCatalog.asset_aliases?.[item.collection] ?? {}).filter(([name, local]) => item.assets?.[name] && name !== local);
        if (localCollection || mappings.length) {
          const names = el('details', undefined, 'result-aliases'); names.append(el('summary', 'Names in this result'));
          if (localCollection) names.append(el('p', `${localCollection} → ${item.collection}`, 'hint'));
          for (const [name, local] of mappings) names.append(el('p', `${local} → ${name}`, 'hint'));
          body.append(names);
        }
      }
      const sources = el('div', undefined, 'sources');
      for (const id of entry.seen_in) sources.append(el('span', selected.catalogs.find(c => c.id === id)?.name ?? id, 'source'));
      body.append(sources);
      if (href || item.geometry || previewAssets(item, base).length) {
        const links = viewerLinks(href); const actions = el('div', undefined, 'scene-actions');
        actions.append(button('Preview on map', () => preview(entry, href), 'primary'));
        body.append(actions);
        if (links) {
          const viewers = el('details'); viewers.append(el('summary', 'Open in another viewer'));
          const choices = el('div', undefined, 'scene-actions');
          choices.append(link('STAC Map ↗', links.map), link('STAC Browser ↗', links.browser));
          viewers.append(choices); body.append(viewers);
        }
      } else body.append(el('p', 'This scene has no public record link for the map or browser.', 'hint'));
      if (assets.length) {
        const details = el('details'); details.append(el('summary', 'Scene assets'));
        const list = el('ul', undefined, 'asset-list');
        for (const [name, asset] of assets) {
          const row = el('li'); const assetUrl = httpUrl(asset.href, base);
          row.append(assetUrl ? link(asset.title ?? name, assetUrl) : el('span', asset.title ?? name));
          if (asset.roles?.length) row.append(el('span', asset.roles.join(', '), 'hint'));
          list.append(row);
        }
        details.append(list); body.append(details);
      }
      card.append(body); $('scenes').append(card);
    }
    if (!response.items.length) {
      $('scenes').append(el('p', response.metadata.catalogs_succeeded ? 'No scenes matched. Try a wider date range or another area.' : 'No catalog returned results. Check the catalog details above or try another catalog.', 'hint'), button('Adjust search', () => document.dispatchEvent(new Event('playground:edit-search'))));
    }
    const pages = Math.ceil(response.items.length / pageSize);
    $('pagination').hidden = pages <= 1; $('previous').disabled = page === 0; $('next').disabled = page >= pages - 1;
    $('page-label').textContent = `Page ${page + 1} of ${pages}`;
  }
  function collapseResults(collapsed) {
    if (!collapsed) document.dispatchEvent(new Event('playground:results-open'));
    $('result-body').hidden = collapsed; $('toggle-results').textContent = collapsed ? 'Show results' : 'Hide results';
    $('toggle-results').setAttribute('aria-expanded', String(!collapsed));
    $('result-content').classList.toggle('collapsed', collapsed);
  }
  $('toggle-results').addEventListener('click', () => collapseResults(!$('result-body').hidden));
  $('previous').addEventListener('click', () => { if (page > 0) { page--; cards(); } });
  $('next').addEventListener('click', () => { if ((page + 1) * pageSize < response.items.length) { page++; cards(); } });
  $('close-map').addEventListener('click', () => { closeMap(); collapseResults(false); $('toggle-results').focus({ preventScroll: true }); });
  return {
    collapse: () => collapseResults(true),
    preview(index) {
      const entry = response?.items[index]; if (!entry) return;
      const href = itemUrl(entry, selected.catalogs);
      preview(entry, href);
    },
    clear() { response = undefined; selected = undefined; overview.scenes([]); closeMap(); $('result-content').hidden = true; $('empty').hidden = false; $('scenes').replaceChildren(); $('response').textContent = ''; },
    render(result, snapshot) {
      response = result; selected = snapshot; page = 0; overview.scenes(result.items); closeMap();
      const { metadata, items } = result;
      $('empty').hidden = true; $('result-content').hidden = false; collapseResults(false);
      $('result-title').textContent = `${items.length} ${items.length === 1 ? 'scene' : 'scenes'} found`;
      $('result-query').textContent = `${snapshot.areaName} · ${snapshot.query.collections.join(', ') || 'All collections'} · ${snapshot.query.datetime ? snapshot.query.datetime.split('/').map(date => date === '..' ? 'Any date' : new Intl.DateTimeFormat('en', { dateStyle: 'medium', timeZone: 'UTC' }).format(new Date(date))).join(' – ') : 'Any date'}`;
      $('result-version').textContent = `SuperSTAC ${metadata.superstac_version}`;
      $('summary').replaceChildren(el('span', `${metadata.catalogs_succeeded} of ${metadata.catalogs_queried} catalogs responded`), el('span', `${metadata.duplicates_removed} duplicate ${metadata.duplicates_removed === 1 ? 'record' : 'records'} combined`));
      $('catalog-status').replaceChildren();
      for (const catalog of snapshot.catalogs.filter(c => c.enabled)) {
        const failure = metadata.failures.find(f => f.catalog_id === catalog.id);
        const count = items.filter(entry => entry.seen_in.includes(catalog.id)).length;
        const note = el('div', undefined, `catalog-note${failure ? ' failure' : ''}`);
        note.append(el('span', failure ? `${catalog.name} could not complete this search.` : `${catalog.name}: ${count} ${count === 1 ? 'scene' : 'scenes'}`));
        if (failure) { const details = el('details'); details.append(el('summary', 'Error details'), el('pre', failure.reason)); note.append(details); }
        $('catalog-status').append(note);
      }
      if (metadata.unsupported_collections?.length) $('catalog-status').append(el('p', `Unsupported collections: ${metadata.unsupported_collections.join(', ')}`, 'hint'));
      cards(); $('response').textContent = JSON.stringify(result, null, 2);
    },
  };
}
