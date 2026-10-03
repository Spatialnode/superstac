'use client';

import { RootProvider } from 'fumadocs-ui/provider/next';
import { usePathname } from 'next/navigation';
import type { ReactNode } from 'react';
import config from '@/versions.json';

const { latest, versions } = config;

export function DocsProvider({ children }: { children: ReactNode }) {
  const parts = usePathname().split('/');
  const requested = parts[parts.indexOf('docs') + 1];
  const version = versions.includes(requested) ? requested : latest;

  return (
    <RootProvider
      theme={{ storageKey: 'theme', defaultTheme: 'system', enableSystem: true, disableTransitionOnChange: true }}
      search={{ options: { api: '/superstac/api/search', defaultTag: version } }}
    >
      {children}
    </RootProvider>
  );
}
