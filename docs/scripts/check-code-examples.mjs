// Pure data checks: no browser, WASM build, or catalog requests.
import assert from 'node:assert/strict';
import { codeFiles, codeTokens } from '../public/playground/modules/code-examples.js';
import { configFor, defaults } from '../public/playground/modules/model.js';

const config = configFor(defaults());
config.catalogs[0].collection_aliases = { optical: 'sentinel-2-l2a' };
config.catalogs[0].asset_aliases = { 'sentinel-2-l2a': { rgb: 'visual' } };
const query = {
  collections: ['optical'], bbox: [6, 45, 7, 46],
  datetime: '2024-06-01T00:00:00Z/2024-06-30T23:59:59Z',
  limit: 5, sortby: ['-datetime'], ids: ['scene'],
};
const original = JSON.stringify({ config, query });
const names = {
  JavaScript: ['search.js'], Python: ['search.py'],
  Rust: ['src/main.rs', 'Cargo.toml', 'superstac.yml'],
  CLI: ['search.sh', 'superstac.yml'],
};
for (const [language, expected] of Object.entries(names)) {
  const output = codeFiles(config, query, language);
  assert.deepEqual(output.files.map(file => file.name), expected);
  assert.ok(output.guide.startsWith('/superstac/docs/'));
  for (const file of output.files) {
    const tokens = codeTokens(file.content, file.syntax);
    assert.equal(tokens.map(token => token.text).join(''), file.content);
    assert.ok(tokens.some(token => token.kind));
  }
  assert.match(output.files[0].content, /optical/);
}
assert.equal(JSON.stringify({ config, query }), original, 'Generating code must not change the search');
const nativeConfig = codeFiles(config, query, 'Rust').files[2].content;
assert.equal(nativeConfig, codeFiles(config, query, 'CLI').files[1].content);
assert.match(nativeConfig, /"collection_aliases":/);
assert.match(nativeConfig, /"asset_aliases":/);
assert.match(nativeConfig, /"deduplicate_items": true/);
assert.match(codeFiles(config, query, 'JavaScript').files[0].content, /@spatialnode\/superstac-wasm/);
assert.match(codeFiles(config, query, 'Python').files[0].content, /"deduplicate_items": True/);
assert.match(codeFiles(config, query, 'CLI').files[0].content, /--sortby='-datetime'/);

const geometry = { collections: [], intersects: { type: 'Point', coordinates: [6, 45] } };
assert.throws(() => codeFiles(config, geometry, 'CLI'), /does not support GeoJSON/);
for (const language of ['JavaScript', 'Python', 'Rust']) {
  assert.match(codeFiles(config, geometry, language).files[0].content, /Point/);
}
const unusual = { ...query, ids: ['it\'s "# a scene </span><script> with $HOME and `ticks`'] };
assert.match(codeFiles(config, unusual, 'Rust').files[0].content, /r##"/);
assert.ok(codeFiles(config, unusual, 'CLI').files[0].content.includes("'\\''"));
for (const language of ['javascript', 'python', 'rust', 'bash', 'yaml', 'toml']) {
  const source = unusual.ids[0] + '\n# comment\n"quoted\\\"value"';
  assert.equal(codeTokens(source, language).map(token => token.text).join(''), source);
}
console.log('Code examples passed: four languages, settings, aliases, geometry support, escaping, and lossless coloring.');
