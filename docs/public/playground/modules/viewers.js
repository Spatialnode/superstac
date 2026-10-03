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
export function thumbnailCandidates(item, base) {
  const assets = Object.entries(item.assets ?? {});
  const rendered = ([key, asset]) => key === 'rendered_preview' || asset.roles?.includes('overview');
  const isThumbnail = ([key, asset]) => key === 'thumbnail' || asset.roles?.includes('thumbnail');
  const usable = ([key, asset]) => !/tiff/i.test(asset.type ?? '') && !/\.tiff?(?:[?#]|$)/i.test(asset.href ?? '') &&
    (asset.type?.match(/^image\/(jpeg|png|webp|gif|avif)(?:;|$)/i) || /\.(jpe?g|png|webp|gif|avif)(?:[?#]|$)/i.test(asset.href ?? '') || key === 'thumbnail' || key === 'rendered_preview');
  const pictures = assets.filter(pair => pair[1] && usable(pair));
  const ordered = [...pictures.filter(isThumbnail), ...pictures.filter(rendered), ...pictures];
  const links = (item.links ?? []).filter(link => link.rel === 'preview' &&
    (link.type?.match(/^image\/(jpeg|png|webp|gif|avif)(?:;|$)/i) || /\.(jpe?g|png|webp|gif|avif)(?:[?#]|$)/i.test(link.href ?? '')));
  return [...new Set([...ordered.map(([, asset]) => httpUrl(asset.href, base)), ...links.map(link => httpUrl(link.href, base))].filter(Boolean))];
}

export function thumbnail(item, base) {
  return thumbnailCandidates(item, base)[0] ?? null;
}

export function previewAssets(item, base) {
  return Object.entries(item.assets ?? {}).flatMap(([id, asset]) => {
    const href = httpUrl(asset.href, base);
    if (!href) return [];
    const type = asset.type ?? '';
    const cog = /tiff/i.test(type) || /\.tiff?(?:[?#]|$)/i.test(href);
    const picture = id === 'rendered_preview' || /^image\/(jpeg|png|webp)/i.test(type) || /\.(jpe?g|png|webp)(?:[?#]|$)/i.test(href);
    return cog || picture ? [{ id, href, cog, title: asset.title ?? id, thumbnail: id === 'thumbnail' || asset.roles?.includes('thumbnail') }] : [];
  });
}
