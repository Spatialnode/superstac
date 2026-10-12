import { $, el, button } from './dom.js';
import { aliases } from './model.js';

export function aliasValues(collectionRows, assetRows) {
  const collection_aliases = Object.create(null), asset_aliases = Object.create(null);
  for (const [name, local] of collectionRows) {
    if (!name && !local) continue;
    if (!name || !local) throw new Error('Enter both names for each collection alias.');
    if (Object.hasOwn(collection_aliases, name)) throw new Error(`The collection name “${name}” is used twice.`);
    collection_aliases[name] = local;
  }
  for (const [collection, name, local] of assetRows) {
    if (!collection && !name && !local) continue;
    if (!collection || !name || !local) throw new Error('Enter a collection and both names for each asset alias.');
    asset_aliases[collection] ??= Object.create(null);
    if (Object.hasOwn(asset_aliases[collection], name)) throw new Error(`The asset name “${name}” is used twice in “${collection}”.`);
    asset_aliases[collection][name] = local;
  }
  return { collection_aliases, asset_aliases };
}

export function aliasEditor() {
  let rawEdited = false;
  const rows = id => [...$(id).children].map(row => [...row.querySelectorAll('input')].map(input => input.value.trim()));
  const readRows = () => aliasValues(rows('collection-alias-rows'), rows('asset-alias-rows'));
  function explain(config) {
    const preview = $('alias-preview'); preview.replaceChildren();
    for (const [name, local] of Object.entries(config.collection_aliases)) {
      preview.append(el('p', `Search for “${name}” → this catalog receives “${local}”.`));
    }
    for (const [collection, pairs] of Object.entries(config.asset_aliases)) for (const [name, local] of Object.entries(pairs)) {
      preview.append(el('p', `In “${collection}” results, “${local}” appears as “${name}” when names are normalized.`));
    }
    if (!preview.children.length) preview.append(el('p', 'Keep one set of names in your app, even when catalogs use different names.'));
  }
  function sync() {
    rawEdited = false;
    try {
      const config = readRows();
      $('collection-aliases').value = JSON.stringify(config.collection_aliases, null, 2);
      $('asset-aliases').value = JSON.stringify(config.asset_aliases, null, 2);
      explain(config);
    } catch (error) { $('alias-preview').textContent = error.message; }
  }
  function row(id, values) {
    const node = el('div', undefined, 'alias-row');
    const labels = id === 'collection-alias-rows' ? ['Name in your app', 'Name in this catalog'] : ['Collection name in your app', 'Asset name in your app', 'Asset name in this catalog'];
    values.forEach((value, index) => {
      const label = el('label', labels[index]); const input = el('input'); input.value = value;
      input.placeholder = (values.length === 2 ? ['optical', 'sentinel-2-l2a'] : ['optical', 'rgb', 'visual'])[index];
      input.addEventListener('input', sync); label.append(input); node.append(label);
    });
    const remove = button('×', () => { node.remove(); sync(); }, 'quiet'); remove.setAttribute('aria-label', 'Remove alias'); node.append(remove); $(id).append(node);
  }
  function draw(config) {
    $('collection-alias-rows').replaceChildren(); $('asset-alias-rows').replaceChildren();
    Object.entries(config.collection_aliases).forEach(pair => row('collection-alias-rows', pair));
    Object.entries(config.asset_aliases).forEach(([collection, pairs]) => Object.entries(pairs).forEach(pair => row('asset-alias-rows', [collection, ...pair])));
    explain(config);
  }
  function load(config) {
    rawEdited = false; draw(config);
    $('collection-aliases').value = JSON.stringify(config.collection_aliases, null, 2);
    $('asset-aliases').value = JSON.stringify(config.asset_aliases, null, 2);
  }
  $('add-collection-alias').addEventListener('click', () => { row('collection-alias-rows', ['', '']); $('collection-alias-rows').lastElementChild.querySelector('input').focus(); });
  $('add-asset-alias').addEventListener('click', () => { row('asset-alias-rows', ['', '', '']); $('asset-alias-rows').lastElementChild.querySelector('input').focus(); });
  $('fill-alias-example').addEventListener('click', () => load({ collection_aliases: { optical: 'sentinel-2-l2a' }, asset_aliases: { optical: { rgb: 'visual' } } }));
  for (const id of ['collection-aliases', 'asset-aliases']) $(id).addEventListener('input', () => {
    rawEdited = true;
    try { draw({ collection_aliases: aliases($('collection-aliases').value), asset_aliases: aliases($('asset-aliases').value, true) }); }
    catch { $('alias-preview').textContent = 'Finish editing the JSON to see the name mappings.'; }
  });
  return {
    load,
    read() { return rawEdited ? { collection_aliases: aliases($('collection-aliases').value), asset_aliases: aliases($('asset-aliases').value, true) } : readRows(); },
  };
}
