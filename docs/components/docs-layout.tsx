'use client';

import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { usePathname } from 'next/navigation';
import type { ComponentProps } from 'react';
import { baseOptions } from '@/lib/layout.shared';
import config from '@/versions.json';

const { latest, versions } = config;

export function VersionedDocsLayout({ tree, children }: Pick<ComponentProps<typeof DocsLayout>, 'tree' | 'children'>) {
  const parts = usePathname().split('/');
  const requested = parts[parts.indexOf('docs') + 1];
  const version = versions.includes(requested) ? requested : latest;

  return <DocsLayout {...baseOptions(version)} tree={tree}>{children}</DocsLayout>;
}
