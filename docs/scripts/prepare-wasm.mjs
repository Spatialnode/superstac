// Copy a previously built package; never compile Rust as part of local docs startup.
import { copyFile, mkdir, readFile } from 'node:fs/promises';
const source = new URL('../../crates/wasm/pkg/', import.meta.url);
const destination = new URL('../public/wasm/', import.meta.url);
const files = ['superstac_wasm.js', 'superstac_wasm_bg.wasm', 'superstac_wasm.d.ts'];
try {
  await Promise.all(files.map(name => readFile(new URL(name, source))));
} catch {
  console.error('Build the browser package first (see crates/wasm/README.md), or download the wasm-web CI artifact into docs/public/wasm. No Rust build was started.');
  process.exit(1);
}
await mkdir(destination, { recursive: true });
for (const name of files) await copyFile(new URL(name, source), new URL(name, destination));
console.log('Browser package copied to docs/public/wasm.');
