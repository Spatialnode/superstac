import init, { SuperSTAC, type SearchResponse, type CatalogCollections } from '../pkg/superstac_wasm.js';
await init();
const client = new SuperSTAC({ catalogs: [{ id: 'test', url: 'https://example.com' }] });
const response: SearchResponse = await client.search({ bbox: [0, 0, 1, 1], limit: 2 });
const collections: CatalogCollections[] = await client.listCollections();
console.log(response.metadata.total_items, collections[0].catalog_id);
// @ts-expect-error limit must be a number
client.search({ limit: '2' });
client.free();
