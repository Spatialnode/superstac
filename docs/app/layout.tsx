import type { Metadata } from 'next';
import localFont from 'next/font/local';
import { RootProvider } from 'fumadocs-ui/provider/next';
import './global.css';

// Match Spatialnode's Inter UI and Crimson Pro display typography, served locally.
const inter = localFont({
  src: '../assets/fonts/inter-latin.woff2',
  variable: '--font-inter',
  weight: '100 900',
  display: 'swap',
});
const crimson = localFont({
  src: '../assets/fonts/crimson-pro-latin.woff2',
  variable: '--font-crimson-pro',
  weight: '200 900',
  display: 'swap',
});

export const metadata: Metadata = {
  metadataBase: new URL('https://spatialnode.com'),
  title: { default: 'SuperSTAC — Many catalogs. One search.', template: '%s | SuperSTAC' },
  description: 'Search across STAC catalogs through one interface, with Python, Rust, or the command line.',
  icons: { icon: '/superstac/favicon.svg' },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${inter.variable} ${crimson.variable}`}>
        <RootProvider theme={{ storageKey: 'theme', defaultTheme: 'system', enableSystem: true, disableTransitionOnChange: true }} search={{ options: { api: '/superstac/api/search' } }}>
          {children}
        </RootProvider>
      </body>
    </html>
  );
}
