import { source } from '@/lib/source';
import { createFromSource } from 'fumadocs-core/search/server';
import config from '@/versions.json';

const { latest, versions } = config;

const search = createFromSource(source, {
  language: 'english',
  buildIndex: (page) => ({
    id: page.url,
    title: page.data.title,
    description: page.data.description,
    structuredData: page.data.structuredData,
    url: page.url,
    tag: page.slugs[0],
  }),
});

export function GET(request: Request) {
  const url = new URL(request.url);
  const version = url.searchParams.get('tag') ?? latest;
  if (!versions.includes(version)) return Response.json({ error: 'Unknown docs version' }, { status: 400 });
  url.searchParams.set('tag', version);
  return search.GET(new Request(url, request));
}
