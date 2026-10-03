import { $, el, button, errorMessage } from './dom.js';
import { providerIcon } from './provider-icon.js';
import { aliasEditor } from './alias-editor.js';
import { identifier, catalogUrl, httpUrl } from './model.js';

export function catalogEditor(getState, changed, isBusy, badge) {
  let editingCatalog, editingProvider;
  const aliasFields = aliasEditor();
  const showError = (id, error) => { $(id).textContent = errorMessage(error); $(id).hidden = false; };
  function openCatalog(catalog, adding = false) {
    if (isBusy()) return;
    editingCatalog = adding ? undefined : catalog?.id;
    $('catalog-form').reset(); delete $('catalog-id').dataset.manual; $('catalog-error').hidden = true;
    $('catalog-dialog-title').textContent = catalog && !adding ? 'Edit catalog' : 'Add catalog';
    for (const key of ['name', 'id', 'url']) $(`catalog-${key}`).value = catalog?.[key] ?? '';
    $('catalog-provider').replaceChildren(new Option('Unassigned', ''), ...getState().providers.map(p => new Option(p.name, p.id)));
    $('catalog-provider').value = catalog?.provider ?? '';
    aliasFields.load({ collection_aliases: catalog?.collection_aliases ?? {}, asset_aliases: catalog?.asset_aliases ?? {} });
    $('catalog-dialog').showModal();
  }
  function openProvider(provider) {
    if (isBusy()) return;
    editingProvider = provider?.id;
    $('provider-form').reset(); delete $('provider-id').dataset.manual; $('provider-error').hidden = true;
    $('provider-dialog-title').textContent = provider ? 'Edit provider' : 'Add provider';
    $('provider-icon').value = provider?.icon ?? '';
    $('provider-name').value = provider?.name ?? ''; $('provider-id').value = provider?.id ?? '';
    $('provider-dialog').showModal();
  }
  function render() {
    const state = getState();
    $('catalog-list').replaceChildren(); $('active-catalogs').replaceChildren();
    for (const catalog of state.catalogs) {
      if (catalog.enabled) {
        const source = el('div', undefined, 'active-source');
        const info = el('div'); info.append(el('strong', catalog.name), badge(catalog));
        source.append(providerIcon(state.providers.find(p => p.id === catalog.provider)), info); $('active-catalogs').append(source);
      }
      const card = el('div', undefined, 'catalog-entry');
      const label = el('label', undefined, 'check');
      const check = el('input'); check.type = 'checkbox'; check.checked = catalog.enabled; check.disabled = isBusy();
      check.addEventListener('change', () => { catalog.enabled = check.checked; changed(); });
      label.append(check, providerIcon(state.providers.find(p => p.id === catalog.provider)), el('strong', catalog.name)); card.append(label);
      card.append(el('p', state.providers.find(p => p.id === catalog.provider)?.name ?? 'Unassigned', 'hint'), el('p', catalog.url, 'catalog-url'));
      const actions = el('div', undefined, 'toolbar');
      actions.append(button('Edit', () => openCatalog(catalog), 'quiet'), button('Remove', () => {
        if (isBusy()) return;
        state.catalogs = state.catalogs.filter(c => c !== catalog); changed();
      }, 'quiet'));
      card.append(badge(catalog), actions); $('catalog-list').append(card);
    }
    if (!state.catalogs.length) $('catalog-list').append(el('p', 'Add a catalog to start searching.', 'hint'));
    $('provider-list').replaceChildren();
    for (const provider of state.providers) {
      const row = el('div', undefined, 'provider-entry'); row.append(providerIcon(provider), el('span', provider.name));
      const actions = el('div', undefined, 'toolbar');
      actions.append(button('Edit', () => openProvider(provider), 'quiet'), button('Remove', () => {
        if (isBusy()) return;
        state.providers = state.providers.filter(p => p !== provider);
        state.catalogs.forEach(c => { if (c.provider === provider.id) c.provider = ''; }); changed();
      }, 'quiet')); row.append(actions); $('provider-list').append(row);
    }
    document.querySelectorAll('#catalog-list button, #provider-list button, #browse-index, #add-catalog, #add-provider, #discover, #check-health').forEach(b => { b.disabled = isBusy(); });
  }
  $('add-catalog').addEventListener('click', () => openCatalog());
  $('add-provider').addEventListener('click', () => openProvider());
  document.querySelectorAll('[data-close]').forEach(b => b.addEventListener('click', () => $(b.dataset.close).close()));
  $('catalog-form').addEventListener('submit', event => {
    event.preventDefault(); if (isBusy()) return;
    try {
      const state = getState(); const id = identifier($('catalog-id').value.trim(), 'Catalog ID');
      if (state.catalogs.some(c => c.id === id && c.id !== editingCatalog)) throw new Error('A catalog already uses this ID. Choose another.');
      const existing = state.catalogs.find(c => c.id === editingCatalog);
      const name = $('catalog-name').value.trim(); if (!name) throw new Error('Enter a catalog name.');
      const catalog = { id, name, url: catalogUrl($('catalog-url').value.trim()), provider: $('catalog-provider').value, enabled: existing?.enabled ?? true,
        ...aliasFields.read() };
      if (existing) state.catalogs.splice(state.catalogs.indexOf(existing), 1, catalog); else state.catalogs.push(catalog);
      $('catalog-dialog').close(); changed();
    } catch (error) { showError('catalog-error', error); }
  });
  $('provider-form').addEventListener('submit', event => {
    event.preventDefault(); if (isBusy()) return;
    try {
      const state = getState(); const id = identifier($('provider-id').value.trim(), 'Provider ID');
      const icon = $('provider-icon').value.trim(); if (icon && !httpUrl(icon)) throw new Error('Use an HTTP or HTTPS image URL.');
      const name = $('provider-name').value.trim(); if (!name) throw new Error('Enter a provider name.');
      if (state.providers.some(p => p.id === id && p.id !== editingProvider)) throw new Error('A provider already uses this ID. Choose another.');
      const existing = state.providers.find(p => p.id === editingProvider);
      if (existing) {
        state.catalogs.forEach(c => { if (c.provider === editingProvider) c.provider = id; });
        Object.assign(existing, { id, name, icon });
      } else state.providers.push({ id, name, icon });
      $('provider-dialog').close(); changed();
    } catch (error) { showError('provider-error', error); }
  });
  for (const kind of ['catalog', 'provider']) $(kind + '-name').addEventListener('input', () => {
    if ((kind === 'catalog' ? editingCatalog : editingProvider) || $(kind + '-id').dataset.manual) return;
    $(kind + '-id').value = $(kind + '-name').value.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  });
  for (const kind of ['catalog', 'provider']) $(kind + '-id').addEventListener('input', () => { $(kind + '-id').dataset.manual = 'true'; });
  return { render, addFromIndex: catalog => openCatalog(catalog, true) };
}
