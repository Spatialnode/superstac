// Plain data and validation, separate from rendering and the WASM boundary.
export const AREAS = {
  alps: { name: 'French Alps', bbox: [6, 45, 7, 46] },
  barcelona: { name: 'Barcelona coast', bbox: [1.9, 41.2, 2.4, 41.6] },
  nairobi: { name: 'Nairobi', bbox: [36.6, -1.5, 37, -1.1] },
};
export const DEFAULT_SETTINGS = {
  deduplicate_items: true, unify_response: true, max_concurrent_catalogs: 8,
  per_catalog_timeout_seconds: 30, max_retry_attempts: 2, max_items_per_catalog: 1000,
};
export function defaults() {
  return {
    providers: [{ id: 'element84', name: 'Element 84' }, { id: 'microsoft', name: 'Microsoft' }],
    catalogs: [
      { id: 'earth-search', name: 'Earth Search', provider: 'element84', url: 'https://earth-search.aws.element84.com/v1', enabled: true, collection_aliases: {}, asset_aliases: {} },
      { id: 'planetary-computer', name: 'Planetary Computer', provider: 'microsoft', url: 'https://planetarycomputer.microsoft.com/api/stac/v1', enabled: true, collection_aliases: {}, asset_aliases: {} },
    ], settings: { ...DEFAULT_SETTINGS },
  };
}
export function identifier(value, label = 'ID') {
  if (!/^[A-Za-z0-9_-]+$/.test(value)) throw new Error(`${label} can contain letters, numbers, hyphens, and underscores.`);
  return value;
}
export function httpUrl(value, base) {
  if (typeof value !== 'string' || !value.trim()) return null;
  try { const u = new URL(value, base); return ['http:', 'https:'].includes(u.protocol) ? u.href : null; }
  catch { return null; }
}
export function catalogUrl(value) {
  const parsed = new URL(value);
  if (!httpUrl(value) || parsed.username || parsed.password || parsed.search || parsed.hash) throw new Error('Use an HTTP or HTTPS catalog URL without credentials, query parameters, or a fragment.');
  return parsed.href.replace(/\/$/, '');
}
export function aliases(text, nested = false) {
  const value = JSON.parse(text.trim() || '{}');
  const record = obj => obj && typeof obj === 'object' && !Array.isArray(obj);
  if (!record(value) || !Object.values(value).every(v => nested ? record(v) && Object.values(v).every(s => typeof s === 'string') : typeof v === 'string')) {
    throw new Error(nested ? 'Asset aliases must map a collection to an object of name pairs.' : 'Collection aliases must be an object of name pairs.');
  }
  return value;
}
export function configFor(state) {
  const selected = state.catalogs.filter(c => c.enabled);
  if (!selected.length) throw new Error('Select at least one catalog in Catalogs & providers.');
  return { catalogs: selected.map(({ id, url, collection_aliases, asset_aliases }) => ({ id, url, collection_aliases, asset_aliases })), settings: { ...state.settings } };
}
export function splitList(text) { return text.split(/[,\n]/).map(v => v.trim()).filter(Boolean); }
export function queryFor(values) {
  const query = { collections: splitList(values.collections), limit: Number(values.limit) };
  if (!Number.isInteger(query.limit) || query.limit < 1 || query.limit > 500) throw new Error('Choose between 1 and 500 scenes per catalog.');
  if (values.from && values.to && values.from > values.to) throw new Error('The end date must be on or after the start date.');
  if (values.from || values.to) query.datetime = `${values.from ? values.from + 'T00:00:00Z' : '..'}/${values.to ? values.to + 'T23:59:59Z' : '..'}`;
  if (values.geometry.trim()) {
    const geometry = JSON.parse(values.geometry);
    if (!geometry || !['Point', 'MultiPoint', 'LineString', 'MultiLineString', 'Polygon', 'MultiPolygon', 'GeometryCollection'].includes(geometry.type)) throw new Error('Use a GeoJSON geometry, such as a Polygon, rather than a Feature.');
    query.intersects = geometry;
  } else if (values.area !== 'anywhere') {
    const bbox = values.area === 'custom' ? values.bbox.map(v => v.trim() === '' ? NaN : Number(v)) : AREAS[values.area]?.bbox;
    if (!bbox || bbox.length !== 4 || bbox.some(v => !Number.isFinite(v)) || bbox[0] < -180 || bbox[2] > 180 || bbox[1] < -90 || bbox[3] > 90 || bbox[0] >= bbox[2] || bbox[1] >= bbox[3]) throw new Error('Enter a valid bounding box: west < east, south < north, with longitude −180 to 180 and latitude −90 to 90.');
    query.bbox = bbox;
  }
  const ids = splitList(values.ids); if (ids.length) query.ids = ids;
  const sortby = splitList(values.sortby); if (sortby.length) query.sortby = sortby;
  return query;
}
export function example(config, query) {
  return `import init, { SuperSTAC } from 'superstac-wasm';\n\nawait init();\n\n// Configure the catalogs your application searches.\nconst client = new SuperSTAC(${JSON.stringify(config, null, 2)});\n\ntry {\n  const result = await client.search(${JSON.stringify(query, null, 2).replaceAll('\n', '\n  ')});\n\n  for (const { item, seen_in } of result.items) {\n    console.log(item.id, item.properties.datetime, seen_in);\n  }\n\n  // A catalog can fail while others still return scenes.\n  for (const failure of result.metadata.failures) {\n    console.warn(failure.catalog_id, failure.reason);\n  }\n} finally {\n  client.free();\n}`;
}
