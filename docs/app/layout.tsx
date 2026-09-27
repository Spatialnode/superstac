import type { Metadata } from 'next';
import { RootProvider } from 'fumadocs-ui/provider/next';
import './global.css';

export const metadata: Metadata = {
  metadataBase: new URL('https://spatialnode.com'),
  title: { default: 'SuperSTAC — Many catalogs. One search.', template: '%s | SuperSTAC' },
  description: 'Search across STAC catalogs through one interface, with Python, Rust, or the command line. A Spatialnode open-source project.',
  icons: { icon: '/superstac/favicon.svg' },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body>
        <RootProvider theme={{ storageKey: 'superstac-theme' }} search={{ options: { api: '/superstac/api/search' } }}>
          {children}
        </RootProvider>
      </body>
    </html>
  );
}
