import { copyFile, mkdir } from 'node:fs/promises';
await mkdir(new URL('../public/brand/', import.meta.url), { recursive: true });
for (const name of ['superstac-logo.svg', 'superstac-logo-dark.svg']) {
  await copyFile(new URL(`../assets/${name}`, import.meta.url), new URL(`../public/brand/${name}`, import.meta.url));
}
