import { el } from './dom.js';
import { httpUrl } from './model.js';

export function providerIcon(provider) {
  const icon = el('span', undefined, 'provider-icon'); icon.setAttribute('aria-hidden', 'true');
  const fallback = () => icon.replaceChildren(el('span', (provider?.name ?? 'Catalog').split(/\s+/).slice(0, 2).map(w => w[0]).join('').toUpperCase()));
  fallback();
  if (httpUrl(provider?.icon)) {
    const img = el('img'); img.src = provider.icon; img.alt = ''; img.referrerPolicy = 'no-referrer'; img.loading = 'lazy';
    img.addEventListener('error', fallback, { once: true }); icon.replaceChildren(img);
  }
  return icon;
}
