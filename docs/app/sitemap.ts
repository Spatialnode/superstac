import type { MetadataRoute } from 'next';
import { source } from '@/lib/source';

export default function sitemap(): MetadataRoute.Sitemap {
  return [
    { url: 'https://spatialnode.com/superstac' },
    ...source.getPages().map((page) => ({ url: `https://spatialnode.com/superstac${page.url}` })),
  ];
}
