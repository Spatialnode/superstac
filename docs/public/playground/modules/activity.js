import { $, el } from './dom.js';

export const duration = ms => ms < 1000 ? `${Math.round(ms)} ms` : `${(ms / 1000).toFixed(2)} s`;
export function safeRequestUrl(value) {
  try { const url = new URL(value); return `${url.origin}${url.pathname}${url.search ? '?…' : ''}`; }
  catch { return 'Unknown URL'; }
}

// Observe this page's catalog fetches without changing headers, bodies, or responses.
// Map tiles, images and WASM loading are excluded.
export function activityMonitor(healthChanged) {
  const originalFetch = window.fetch.bind(window);
  const health = new Map();
  const entries = [];
  let operation, sequence = 0;
  function render() {
    $('activity-count').textContent = String(entries.length);
    $('activity-empty').hidden = entries.length > 0;
    const filter = $('activity-filter').value;
    $('activity-list').replaceChildren();
    for (const entry of entries.filter(e => filter !== 'errors' || e.failed).slice().reverse()) {
      const row = el('li', undefined, `activity-entry${entry.failed ? ' failed' : ''}`);
      const heading = el('div', undefined, 'activity-heading');
      heading.append(el('strong', entry.title), el('span', entry.ms === undefined ? 'In progress' : duration(entry.ms), 'request-duration'));
      row.append(heading, el('p', entry.detail, 'hint'));
      if (entry.url) row.append(el('code', entry.url, 'request-url'));
      row.append(el('time', new Date(entry.time).toLocaleTimeString([], { hour12: false }), 'activity-time'));
      $('activity-list').append(row);
    }
  }
  function log(title, detail = '', options = {}) {
    const entry = { title, detail, time: Date.now(), ...options }; entries.push(entry);
    if (entries.length > 200) entries.shift(); render(); return entry;
  }
  function setHealth(catalog, status, ms, detail) {
    if (!catalog) return;
    health.set(`${catalog.id}|${catalog.url}`, { status, ms, detail, time: Date.now() });
    healthChanged();
  }
  window.fetch = async (input, init) => {
    const url = typeof input === 'string' ? input : input instanceof URL ? input.href : input.url;
    const scope = operation;
    if (new Headers(init?.headers ?? input?.headers).has('range') || /\.(tiff?|png|jpe?g|webp)$/i.test(new URL(url, location.href).pathname)) return originalFetch(input, init);
    const host = new URL(url, location.href).hostname;
    if (host === 'unpkg.com' || host.endsWith('.openfreemap.org')) return originalFetch(input, init);
    const catalog = scope?.catalogs.find(c => url === c.url || url.startsWith(c.url.replace(/\/$/, '') + '/') || url.startsWith(c.url + '?'));
    // Cross-host pagination made by WASM is still logged, without guessing its catalog.
    if (!scope || (!catalog && !(input instanceof Request && new URL(url).origin !== location.origin))) return originalFetch(input, init);
    const start = performance.now();
    const method = init?.method ?? input?.method ?? 'GET';
    const entry = log(`${method} · ${catalog?.name ?? 'Pagination request'}`, `${scope.name} · Waiting for response`, { url: safeRequestUrl(url), request: true, operation: scope.id });
    try {
      const response = await originalFetch(input, init);
      entry.ms = performance.now() - start; entry.failed = !response.ok;
      entry.detail = `${scope.name} · HTTP ${response.status}`; entry.status = response.status;
      setHealth(catalog, response.ok ? 'Responded' : 'Request failed', entry.ms, `HTTP ${response.status}`);
      render(); return response;
    } catch (error) {
      entry.ms = performance.now() - start; entry.failed = true;
      entry.detail = error.name === 'AbortError' || error.name === 'TimeoutError' ? 'Request cancelled or timed out' : 'Network or CORS error';
      setHealth(catalog, 'Request failed', entry.ms, entry.detail); render(); throw error;
    }
  };
  $('activity-filter').addEventListener('change', render);
  $('clear-activity').addEventListener('click', () => { if (!operation) { entries.length = 0; render(); } });
  $('copy-activity').addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(JSON.stringify(entries, null, 2)); $('activity-note').textContent = 'Activity copied.';
    } catch { $('activity-note').textContent = 'Copy is unavailable in this browser. You can inspect requests in Developer Tools → Network.'; }
  });
  render();
  return {
    log,
    begin(name, catalogs) {
      operation = { id: ++sequence, name, catalogs: structuredClone(catalogs), start: performance.now() };
      $('clear-activity').disabled = true;
      log(name, `${catalogs.length} selected catalogs`, { operation: sequence });
    },
    finish(detail, failed = false) {
      if (!operation) return;
      const ms = performance.now() - operation.start;
      log(`${operation.name} finished`, detail, { ms, failed, operation: operation.id });
      $('operation-time').textContent = `${operation.name}: ${duration(ms)}`;
      operation = undefined; $('clear-activity').disabled = false; return ms;
    },
    outcome(response, catalogs) {
      for (const catalog of catalogs) {
        const failure = response.metadata.failures.find(f => f.catalog_id === catalog.id);
        const previous = health.get(`${catalog.id}|${catalog.url}`);
        setHealth(catalog, failure ? 'Search failed' : 'Search succeeded', previous?.ms, failure ? 'See search response for details' : 'Last search completed');
      }
    },
    badge(catalog) {
      const value = health.get(`${catalog.id}|${catalog.url}`);
      const badge = el('span', value ? `${value.status}${value.ms === undefined ? '' : ` · ${duration(value.ms)}`}` : 'Not checked', 'health-badge');
      badge.dataset.state = !value ? 'unknown' : value.status.includes('failed') ? 'error' : 'ok';
      badge.title = value ? `${value.detail} · ${new Date(value.time).toLocaleTimeString()} · Last response, not continuous uptime` : 'Run a search or check connections';
      return badge;
    },
    async check(catalogs) {
      // Keep manual connection checks light and sequential; no background polling.
      for (const catalog of catalogs) {
        const controller = new AbortController(); const timer = setTimeout(() => controller.abort(), 8000);
        try {
          const response = await fetch(`${catalog.url}/collections?limit=1`, { signal: controller.signal });
          if (response.ok) {
            const body = await response.json();
            if (!Array.isArray(body.collections)) {
              setHealth(catalog, 'Check failed', undefined, 'Response did not contain STAC collections');
              log('Connection check failed', `${catalog.name}: response did not contain STAC collections`, { failed: true, ms: 0 });
            }
          }
        } catch (error) {
          const previous = health.get(`${catalog.id}|${catalog.url}`);
          const detail = error instanceof SyntaxError ? 'The catalog did not return JSON' : error.name === 'AbortError' ? 'Connection check timed out' : 'Connection check failed';
          setHealth(catalog, 'Check failed', previous?.ms, detail);
          log('Connection check failed', `${catalog.name}: ${detail}`, { failed: true, ms: 0 });
        }
        finally { clearTimeout(timer); }
      }
    },
  };
}
