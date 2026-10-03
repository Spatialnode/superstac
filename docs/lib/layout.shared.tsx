import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';
import { Logo } from '@/components/logo';

export function baseOptions(version?: string): BaseLayoutProps {
  const docs = version ? `/docs/${version}` : '/docs';
  return {
    nav: { title: <Logo />, url: '/' },
    githubUrl: 'https://github.com/spatialnode/superstac',
    links: [
      { text: 'Documentation', url: docs },
      { text: 'What’s new', url: `${docs}/releases` },
      { text: 'Python', url: `${docs}/python/overview` },
      { text: 'Rust', url: `${docs}/rust/overview` },
    ],
  };
}
