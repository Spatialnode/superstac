import init, { SuperSTAC } from '@spatialnode/superstac-wasm';

await init();
const client = new SuperSTAC({
  catalogs: [
    {
      id: 'earth-search',
      url: 'https://earth-search.aws.element84.com/v1',
    },
    {
      id: 'microsoft',
      url: 'https://planetarycomputer.microsoft.com/api/stac/v1',
    },
  ],
});

try {
  const result = await client.search({
    collections: ['sentinel-2-l2a'],
    bbox: [6, 45, 7, 46],
    datetime: '2024-06-01T00:00:00Z/2024-06-30T23:59:59Z',
    limit: 5,
  });
  for (const { item, seen_in } of result.items) {
    console.log(item.id, seen_in);
  }
  for (const failure of result.metadata.failures) {
    console.warn(failure.catalog_id, failure.reason);
  }
  const geojson = {
    type: 'FeatureCollection',
    features: result.items.map(({ item }) => item),
  };
  console.log(geojson);
} finally {
  client.free();
}
