import { httpUrl } from './model.js';
export function itemUrl(entry, catalogs) {
  const catalog = catalogs.find(c => c.id === entry.catalog_id);
  const link = entry.item.links?.find(l => l.rel === 'self') ?? entry.item.links?.find(l => l.rel === 'canonical');
  return link?.href ? httpUrl(link.href, catalog ? catalog.url + '/' : undefined) : null;
}
export function viewerLinks(href) {
  if (!httpUrl(href)) return null;
  const map = new URL('https://developmentseed.org/stac-map/');
  map.searchParams.set('href', href);
  // STAC Browser's external route accepts a full URL after /external/.
  return { map: map.href, browser: `https://radiantearth.github.io/stac-browser/#/external/${href}` };
}
// A thumbnail label alone does not establish a browser-displayable format.
function isTiff(asset) {
  let href = asset.href ?? '';
  try { href = decodeURIComponent(href); } catch { /* Keep malformed URLs unchanged. */ }
  return /tiff|geotiff/i.test(asset.type ?? '') || /\.tiff?(?:[?#]|$)/i.test(href);
}
function isBrowserImage(asset) {
  return !isTiff(asset) && (/^image\/(jpeg|png|webp|gif|avif)(?:\s*;|$)/i.test(asset.type ?? '') ||
    /\.(jpe?g|png|webp|gif|avif)(?:[?#]|$)/i.test(asset.href ?? ''));
}

export function thumbnailCandidates(item, base) {
  const assets = Object.entries(item.assets ?? {});
  const rendered = ([key, asset]) => key === 'rendered_preview' || asset.roles?.includes('overview');
  const isThumbnail = ([key, asset]) => key === 'thumbnail' || asset.roles?.includes('thumbnail');
  const pictures = assets.filter(([, asset]) => asset && isBrowserImage(asset));
  const ordered = [...pictures.filter(isThumbnail), ...pictures.filter(rendered), ...pictures];
  const links = (item.links ?? []).filter(link => link?.rel === 'preview' && isBrowserImage(link));
  return [...new Set([...ordered.map(([, asset]) => httpUrl(asset.href, base)), ...links.map(link => httpUrl(link.href, base))].filter(Boolean))];
}

export function thumbnail(item, base) {
  return thumbnailCandidates(item, base)[0] ?? null;
}

export function hotTileJSON(base, assetId) {
  const href = httpUrl(base);
  if (!href) return null;
  const record = new URL(href);
  if (record.origin !== 'https://api.imagery.hotosm.org' || !/^\/stac\/collections\/[^/]+\/items\/[^/]+\/?$/.test(record.pathname)) return null;
  const path = record.pathname.replace(/^\/stac/, '/raster').replace(/\/$/, '');
  const url = new URL(`${path}/WebMercatorQuad/tilejson.json`, record.origin);
  url.searchParams.set('assets', assetId); url.searchParams.set('tilesize', '256'); url.searchParams.set('tile_format', 'png');
  return url.href;
}

export function previewAssets(item, base, assetAliases = {}) {
  return Object.entries(item.assets ?? {}).flatMap(([id, asset]) => {
    const href = httpUrl(asset.href, base);
    if (!href) return [];
    const cog = isTiff(asset);
    const picture = isBrowserImage(asset);
    const tilejson = cog ? hotTileJSON(base, assetAliases[id] ?? id) : null;
    return cog || picture ? [{ id, href, cog, tilejson, title: asset.title ?? id, thumbnail: !cog && (id === 'thumbnail' || asset.roles?.includes('thumbnail')) }] : [];
  });
}
