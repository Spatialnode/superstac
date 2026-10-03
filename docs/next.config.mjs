import { createMDX } from 'fumadocs-mdx/next';
import { readdirSync, readFileSync } from 'node:fs';

const { latest } = JSON.parse(readFileSync(new URL('./versions.json', import.meta.url), 'utf8'));
const withMDX = createMDX();
export default withMDX({
  basePath: '/superstac',
  output: 'standalone',
  reactStrictMode: true,
  async redirects() {
    // Temporary redirects let existing bookmarks follow the latest release.
    const files = readdirSync(new URL(`./content/docs/${latest}/`, import.meta.url), { recursive: true });
    return files.filter((file) => /\.mdx?$/.test(file)).map((file) => {
      const slug = file.replace(/\.mdx?$/, '').replace(/(^|\/)index$/, '');
      const suffix = slug ? `/${slug}` : '';
      return { source: `/docs${suffix}`, destination: `/docs/${latest}${suffix}`, permanent: false };
    });
  },
});
