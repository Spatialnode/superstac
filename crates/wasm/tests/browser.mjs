// Exercise the generated JS package and real browser Fetch against a local STAC server.
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { chromium } from 'playwright';
import assert from 'node:assert/strict';

const calls = [];
let slowClosed = false;
const item = (id) => ({ type: 'Feature', stac_version: '1.0.0', id,
  collection: 'local', geometry: null,
  properties: { datetime: '2024-01-01T00:00:00Z' },
  assets: { B02: { href: 'https://example.com/blue.tif' } }, links: [] });
const server = createServer(async (req, res) => {
  try {
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Headers', 'content-type,x-page');
    res.setHeader('Access-Control-Allow-Methods', 'GET,POST,OPTIONS');
    if (req.method === 'OPTIONS') { res.end(); return; }
    const url = new URL(req.url, 'http://localhost');
    if (url.pathname === '/') {
      res.setHeader('Content-Type', 'text/html'); res.end('<!doctype html><title>WASM tests</title>'); return;
    }
    if (url.pathname.startsWith('/pkg/')) {
      const name = url.pathname.slice(5);
      if (!/^[\w.-]+$/.test(name)) { res.writeHead(404).end(); return; }
      res.setHeader('Content-Type', name.endsWith('.wasm') ? 'application/wasm' : 'text/javascript');
      res.end(await readFile(new URL(`../pkg/${name}`, import.meta.url))); return;
    }
    let raw = '';
    for await (const chunk of req) raw += chunk;
    const body = raw ? JSON.parse(raw) : undefined;
    calls.push({ path: req.url, method: req.method, body, header: req.headers['x-page'] });
    res.setHeader('Content-Type', 'application/json');
    const send = (features, links = []) => res.end(JSON.stringify({ type: 'FeatureCollection', features, links }));
    if (url.pathname.startsWith('/slow/')) { res.on('close', () => { slowClosed = true; }); return; }
    if (url.pathname.startsWith('/broken/')) { res.writeHead(503).end('{}'); return; }
    if (url.pathname === '/retry/search' && calls.filter(c => c.path === req.url).length === 1) {
      res.writeHead(503).end('{}'); return;
    }
    if (url.pathname === '/a/collections') {
      res.end(JSON.stringify({ collections: [{ id: 'local', title: 'Local collection' }],
        links: [{ rel: 'next', href: './collections?page=2' }],
        ...(url.search ? { collections: [{ id: 'other' }], links: [] } : {}) })); return;
    }
    if (url.pathname.endsWith('/collections')) { res.end('{"collections":[],"links":[]}'); return; }
    if (url.pathname === '/loop/search') { send([], [{ rel: 'next', href: './search', method: 'POST', body }]); return; }
    if (url.pathname === '/a/search') {
      send([item(url.search ? 'second' : 'shared')], url.search ? [] : [{ rel: 'next', href: './search?page=2' }]); return;
    }
    if (url.pathname === '/post/search') {
      send([item(body?.token ? 'second' : 'shared')], body?.token ? [] : [
        { rel: 'next', href: './search', method: 'POST', body: { token: 'next' }, merge: true, headers: { 'x-page': 'two' } }
      ]); return;
    }
    send([item('shared')]);
  } catch (e) { res.writeHead(500).end(JSON.stringify({ error: String(e) })); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
// A separate page origin exercises CORS and preflight on the fixture requests.
const pageServer = createServer(server.listeners('request')[0]);
await new Promise(resolve => pageServer.listen(0, '127.0.0.1', resolve));
const pageOrigin = `http://127.0.0.1:${pageServer.address().port}`;
let browser;
try {
  browser = await chromium.launch({ ...(process.env.CHROME_PATH ? { executablePath: process.env.CHROME_PATH } : {}) });
  const page = await browser.newPage();
  await page.goto(pageOrigin);
  const result = await page.evaluate(async ({ origin, live }) => {
    const { default: init, SuperSTAC } = await import('/pkg/superstac_wasm.js');
    await init();
    const check = (condition, message) => { if (!condition) throw new Error(message); };
    const catalog = id => ({ id, url: `${origin}/${id}`,
      collection_aliases: { canonical: 'local' }, asset_aliases: { canonical: { blue: 'B02' } } });
    const make = (ids, settings = {}) => new SuperSTAC({ catalogs: ids.map(catalog), settings });
    const client = make(['a', 'b', 'retry', 'broken']);
    const result = await client.search({ collections: ['canonical'], limit: 2 });
    check(result.metadata.total_items === 2, JSON.stringify(result));
    check(result.metadata.catalogs_succeeded === 3 && result.metadata.catalogs_failed === 1, 'partial failures');
    check(result.metadata.duplicates_removed === 2, 'deduplication');
    const shared = result.items.find(i => i.item.id === 'shared');
    check(shared.seen_in.length === 3, 'provenance');
    check(shared.item.collection === 'canonical' && shared.item.assets.blue.href.endsWith('blue.tif'), 'aliases and plain objects');
    check(!('B02' in shared.item.assets), 'asset alias removes original key');
    check(Array.isArray(result.items) && !(shared.item.properties instanceof Map), 'JS serialization');
    const discovered = await client.listCollections();
    check(discovered[0].collections.length === 2 && discovered[0].collections[0].canonical_id === 'canonical', 'paginated discovery');
    check(typeof discovered[3].error === 'string', 'discovery partial failure');
    client.free();
    const capped = make(['a'], { max_items_per_catalog: 1 });
    check((await capped.search({ limit: 20 })).items.length === 1, 'item cap'); capped.free();
    const post = make(['post']);
    check((await post.search({ collections: ['canonical'], limit: 2 })).items.length === 2, 'POST pagination'); post.free();
    const slow = make(['slow', 'b'], { per_catalog_timeout_seconds: 1, max_retry_attempts: 1 });
    const timed = await slow.search({});
    check(timed.metadata.catalogs_succeeded === 1 && timed.metadata.failures[0].reason.includes('timeout'), 'browser timeout'); slow.free();
    const loop = make(['loop'], { max_retry_attempts: 1 });
    check((await loop.search({})).metadata.failures[0].reason.includes('pagination'), 'pagination cycle'); loop.free();
    for (const config of [{ catalogs: [] }, { catalogs: [catalog('a'), catalog('a')] },
      { catalogs: [catalog('a')], settings: { max_retry_attempts: 0 } },
      { catalogs: [{ id: 'a', url: 'file:///tmp/data' }] }]) {
      let threw = false;
      try { new SuperSTAC(config); } catch (e) { threw = e instanceof Error; }
      check(threw, 'invalid config throws Error');
    }
    const raw = make(['b', 'retry'], { deduplicate_items: false, unify_response: false });
    const unmodified = await raw.search({});
    check(unmodified.items.length === 2 && unmodified.items.every(i => i.item.collection === 'local' && i.item.assets.B02), 'disabled normalization and deduplication');
    raw.free();
    const invalid = make(['a']);
    for (const query of [{ collection: ['typo'] }, { limit: 0 }, { bbox: [1, 2] }, { bbox: [0, 0, 1, 1], intersects: { type: 'Point', coordinates: [0, 0] } }]) {
      let threw = false;
      try { await invalid.search(query); } catch (e) { threw = e instanceof Error; }
      check(threw, 'invalid query rejects with Error');
    }
    invalid.free();
    if (live) {
      const liveClient = new SuperSTAC({ catalogs: [{ id: 'earth-search', url: 'https://earth-search.aws.element84.com/v1' }] });
      const response = await liveClient.search({ collections: ['sentinel-2-l2a'], bbox: [6, 49, 7, 50],
        datetime: '2024-01-01T00:00:00Z/2024-01-31T23:59:59Z', limit: 1 });
      check(response.metadata.catalogs_succeeded === 1 && response.items.length === 1, JSON.stringify(response.metadata));
      liveClient.free();
      return { fixtureTests: 'passed', liveItem: response.items[0].item.id };
    }
    return { fixtureTests: 'passed' };
  }, { origin, live: process.env.SUPERSTAC_LIVE_TEST === '1' });
  assert.equal(calls.filter(c => c.path === '/retry/search').length, 3);
  assert.ok(slowClosed, 'timeout aborts the outstanding Fetch request');
  const postPage = calls.find(c => c.path === '/post/search' && c.body?.token);
  assert.deepEqual(postPage.body.collections, ['local']);
  assert.equal(postPage.header, 'two');
  assert.equal(calls.filter(c => c.path === '/a/search?page=2').length, 1, 'cap must stop pagination');
  console.log(result);
} finally {
  await browser?.close();
  pageServer.closeAllConnections();
  await new Promise(resolve => pageServer.close(resolve));
  server.closeAllConnections();
  await new Promise(resolve => server.close(resolve));
}
