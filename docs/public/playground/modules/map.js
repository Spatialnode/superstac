import { $ } from './dom.js';

const CDN = 'https://unpkg.com/maplibre-gl@6.11.2/dist/';
const collection = features => ({ type: 'FeatureCollection', features });
export function boxGeometry([west, south, east, north]) {
  return { type: 'Polygon', coordinates: [[[west, south], [east, south], [east, north], [west, north], [west, south]]] };
}
export function geometryBounds(geometry) {
  const points = [];
  function walk(value) {
    if (!Array.isArray(value)) return;
    if (value.length >= 2 && typeof value[0] === 'number' && typeof value[1] === 'number') {
      if (Number.isFinite(value[0]) && Number.isFinite(value[1])) points.push(value);
    } else value.forEach(walk);
  }
  function geometryPoints(g) { walk(g?.coordinates); g?.geometries?.forEach(geometryPoints); }
  geometryPoints(geometry);
  if (!points.length) return null;
  return points.reduce((b, p) => [Math.min(b[0], p[0]), Math.min(b[1], p[1]), Math.max(b[2], p[0]), Math.max(b[3], p[1])], [Infinity, Infinity, -Infinity, -Infinity]);
}
export function overviewMap(onArea, onScene) {
  let map, ready = false, loading = false, query = {}, items = [], lastQuery = '';
  let timeout, library, cogLoading, currentPreview;
  let previewVersion = 0;
  const style = () => `https://tiles.openfreemap.org/styles/${document.documentElement.dataset.theme === 'dark' ? 'dark' : 'positron'}`;
  function data() {
    if (!ready) return;
    const geometry = query.intersects ?? (query.bbox ? boxGeometry(query.bbox) : null);
    map.getSource('query')?.setData(collection(geometry ? [{ type: 'Feature', properties: {}, geometry }] : []));
    map.getSource('scenes')?.setData(collection(items.flatMap((entry, index) => entry.item.geometry ? [{ type: 'Feature', properties: { index }, geometry: entry.item.geometry }] : [])));
    $('map-legend').textContent = `Blue · search area${items.length ? ' / purple · scene footprints' : ''}`;
  }
  function fit() {
    if (!ready) return;
    const bounds = query.bbox ?? geometryBounds(query.intersects);
    if (bounds) map.fitBounds([[bounds[0], bounds[1]], [bounds[2], bounds[3]]], { padding: 70, duration: 0, maxZoom: 12 });
    else map.jumpTo({ center: [0, 20], zoom: 1.5 });
  }
  function removePreview() {
    if (!map) return;
    if (map.getLayer('asset-preview')) map.removeLayer('asset-preview');
    if (map.getSource('asset-preview')) map.removeSource('asset-preview');
  }
  async function cogProtocol() {
    if (!cogLoading) cogLoading = new Promise((resolve, reject) => {
      const script = document.createElement('script');
      script.src = 'https://unpkg.com/@geomatico/maplibre-cog-protocol@0.5.0/dist/index.js';
      const timer = setTimeout(() => reject(new Error('The imagery reader could not load. Try an external viewer.')), 12000);
      script.onload = () => {
        clearTimeout(timer);
        if (!window.MaplibreCOGProtocol?.cogProtocol) { reject(new Error('The imagery reader is unavailable.')); return; }
        library.addProtocol('cog', window.MaplibreCOGProtocol.cogProtocol); resolve();
      };
      script.onerror = () => { clearTimeout(timer); script.remove(); reject(new Error('The imagery reader could not load.')); };
      document.head.append(script);
    }).catch(error => { cogLoading = undefined; throw error; });
    return cogLoading;
  }
  async function assetPreview(entry, asset) {
    currentPreview = { entry, asset };
    const version = ++previewVersion;
    if (!ready) { $('preview-status').textContent = 'Wait for the map to load, or open an external viewer.'; return; }
    removePreview();
    const bounds = geometryBounds(entry.item.geometry) ?? (entry.item.bbox?.length === 4 ? entry.item.bbox : null);
    if (bounds) map.fitBounds([[bounds[0], bounds[1]], [bounds[2], bounds[3]]], { padding: 90, duration: 0, maxZoom: 13 });
    if (!asset) { $('preview-status').textContent = 'Scene footprint. Choose an image asset to preview it.'; return; }
    try {
      $('preview-status').dataset.state = 'loading'; $('preview-status').textContent = 'Loading asset…';
      if (asset.cog) {
        await cogProtocol(); if (version !== previewVersion || !ready) return;
        map.addSource('asset-preview', { type: 'raster', url: `cog://${asset.href}`, tileSize: 256 });
        $('preview-status').textContent = 'GeoTIFF preview. Availability depends on CORS, range requests, and the file’s encoding.';
      } else {
        if (!bounds) throw new Error('This image has no scene bounds. Open the asset from the scene card.');
        const [w, s, e, n] = bounds;
        map.addSource('asset-preview', { type: 'image', url: asset.href, coordinates: [[w, n], [e, n], [e, s], [w, s]] });
        $('preview-status').textContent = 'Image placed over scene bounds for a quick look. Alignment is approximate.';
      }
      map.addLayer({ id: 'asset-preview', type: 'raster', source: 'asset-preview', paint: { 'raster-opacity': .85, 'raster-fade-duration': 0 } }, 'scene-line');
    } catch (error) { if (version === previewVersion) { $('preview-status').dataset.state = 'error'; $('preview-status').textContent = error.message; } }
  }
  function failure() {
    clearTimeout(timeout); loading = false; ready = false;
    map?.remove(); map = undefined;
    $('map-fallback').hidden = false; $('retry-map').hidden = false;
    $('use-map-area').disabled = true; $('fit-map').disabled = true;
    $('map-message').textContent = 'The map could not load. You can still search with the area controls and open scenes in STAC Map.';
  }
  async function start() {
    if (loading || ready) return; loading = true; $('retry-map').hidden = true;
    $('map-message').textContent = 'Loading map… You can search while it loads.';
    try {
      if (!document.querySelector('[data-map-style]')) {
        const css = document.createElement('link'); css.rel = 'stylesheet'; css.href = CDN + 'maplibre-gl.css'; css.dataset.mapStyle = ''; document.head.append(css);
      }
      // The module is independent of WASM. Search still works if WebGL/CDN access fails.
      let importTimer;
      const lib = await Promise.race([
        import(CDN + 'maplibre-gl.mjs'),
        new Promise((_, reject) => { importTimer = setTimeout(() => reject(new Error('Map load timed out')), 12000); }),
      ]).finally(() => clearTimeout(importTimer));
      library = lib; lib.setWorkerCount(1);
      map = new lib.Map({ container: 'overview-map', style: style(), center: [6.5, 45.5], zoom: 6, attributionControl: true, renderWorldCopies: false, maxTileCacheSize: 64, fadeDuration: 0 });
      map.addControl(new lib.NavigationControl({ showCompass: false }), 'top-right');
      timeout = setTimeout(failure, 15000);
      map.on('style.load', () => {
        clearTimeout(timeout); ready = true; loading = false;
        $('map-fallback').hidden = true; $('use-map-area').disabled = false; $('fit-map').disabled = false;
        for (const id of ['query', 'scenes']) map.addSource(id, { type: 'geojson', data: collection([]) });
        map.addLayer({ id: 'query-fill', type: 'fill', source: 'query', paint: { 'fill-color': '#081b99', 'fill-opacity': .08 } });
        map.addLayer({ id: 'query-line', type: 'line', source: 'query', paint: { 'line-color': '#435be3', 'line-width': 2, 'line-dasharray': [3, 2] } });
        map.addLayer({ id: 'scene-fill', type: 'fill', source: 'scenes', paint: { 'fill-color': '#7c3aed', 'fill-opacity': .18 } });
        map.addLayer({ id: 'scene-line', type: 'line', source: 'scenes', paint: { 'line-color': '#7c3aed', 'line-width': 1.5 } });
        map.addLayer({ id: 'scene-points', type: 'circle', source: 'scenes', filter: ['==', '$type', 'Point'], paint: { 'circle-radius': 6, 'circle-color': '#7c3aed' } });
        data(); fit(); if (currentPreview) assetPreview(currentPreview.entry, currentPreview.asset);
      });
      for (const layer of ['scene-fill', 'scene-points']) {
        map.on('click', layer, event => { const index = Number(event.features?.[0]?.properties.index); if (items[index]) onScene(index); });
        map.on('mouseenter', layer, () => { map.getCanvas().style.cursor = 'pointer'; });
        map.on('mouseleave', layer, () => { map.getCanvas().style.cursor = ''; });
      }
      map.on('sourcedata', event => { if (event.sourceId === 'asset-preview' && event.isSourceLoaded) $('preview-status').dataset.state = 'ready'; });
      map.on('error', event => {
        if (event.sourceId === 'asset-preview' || currentPreview?.asset) { $('preview-status').dataset.state = 'error'; $('preview-status').textContent = 'This asset could not be displayed. Try another asset or an external viewer.'; }
        else $('map-legend').textContent = 'Some map tiles could not load. Search is still available.';
      });
    } catch { failure(); }
  }
  $('retry-map').addEventListener('click', start);
  $('fit-map').addEventListener('click', fit);
  $('use-map-area').addEventListener('click', () => {
    if (!ready) return;
    const b = map.getBounds();
    onArea([Math.max(-180, b.getWest()), Math.max(-90, b.getSouth()), Math.min(180, b.getEast()), Math.min(90, b.getNorth())].map(n => Number(n.toFixed(5))));
  });
  return {
    start,
    preview: assetPreview,
    clearPreview() { previewVersion++; currentPreview = undefined; removePreview(); fit(); },
    query(value) { const signature = JSON.stringify([value.bbox, value.intersects]); query = value; if (signature !== lastQuery) { lastQuery = signature; data(); fit(); } },
    scenes(value) { items = value; data(); },
    resize() { if (map) requestAnimationFrame(() => map.resize()); },
    theme() { if (ready) { ready = false; map.setStyle(style()); } },
  };
}
