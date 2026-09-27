import { createMDX } from 'fumadocs-mdx/next';

const withMDX = createMDX();
export default withMDX({
  basePath: '/superstac',
  output: 'standalone',
  reactStrictMode: true,
});
