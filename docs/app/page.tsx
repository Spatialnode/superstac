import Link from 'next/link';
import { HomeLayout } from 'fumadocs-ui/layouts/home';
import { ArrowRight, ArrowUpRight, Braces, Layers, Terminal, Scan } from 'lucide-react';
import { baseOptions } from '@/lib/layout.shared';
import { FederationMap } from '@/components/federation-map';
import { CodePreview } from '@/components/code-preview';
import { PoweredBySpatialnode } from '@/components/powered-by-spatialnode';
import type { Metadata } from 'next';
import styles from './landing.module.css';

const socialTitle = 'SuperSTAC — Many catalogs. One search.';
const socialDescription = 'Find satellite imagery across catalogs, combine results, and save study-area metadata for repeated searches with fewer API requests.';
const socialImage = {
  url: 'https://spatialnode.com/superstac/superstac-og.png',
  width: 1200,
  height: 630,
  alt: 'SuperSTAC — Many catalogs. One search. Find satellite imagery across multiple sources.',
};

export const metadata: Metadata = {
  description: socialDescription,
  alternates: { canonical: '/superstac' },
  openGraph: {
    type: 'website',
    url: 'https://spatialnode.com/superstac',
    siteName: 'SuperSTAC by Spatialnode',
    title: socialTitle,
    description: socialDescription,
    images: [{ ...socialImage, type: 'image/png' }],
  },
  twitter: {
    card: 'summary_large_image',
    title: socialTitle,
    description: socialDescription,
    images: [socialImage],
  },
};

const guides = [
  { icon: Scan, title: 'Browser / WASM', text: 'Search live catalogs from a JavaScript or TypeScript app.', href: '/docs/wasm/overview' },
  { icon: Braces, title: 'Python', text: 'Search from scripts and notebooks with the sync or async client.', href: '/docs/python/overview' },
  { icon: Layers, title: 'Rust', text: 'Add the search engine to a Rust application.', href: '/docs/rust/overview' },
  { icon: Terminal, title: 'Command line', text: 'List collections, search catalogs, and export results as JSON.', href: '/docs/cli/overview' },
];

export default function Home() {
  return (
    <HomeLayout {...baseOptions()}>
      <div className={styles.landing} id="main-content">
        <section className={styles.hero} aria-labelledby="home-title">
          <Link href="/docs/releases" className={styles.release}>v0.3 · What’s new <ArrowRight size={13} aria-hidden="true" /></Link>
          <h1 id="home-title">Many catalogs.<br />One search.</h1>
          <p>Search satellite imagery across STAC catalogs.<br className={styles.desktopBreak} /> Get combined results, with a source for every scene.</p>
          <div className={styles.actions}>
            <a href="/superstac/playground" className={styles.primary}>Open playground <ArrowRight size={16} aria-hidden="true" /></a>
            <Link href="/docs/start/quickstart" className={styles.secondary}>Get started</Link>
          </div>
          <p className={styles.heroNote}>Open source. Built in Rust. Use it from Python, JavaScript, Rust, or the CLI.</p>
        </section>

        <section className={styles.example} aria-labelledby="example-title">
          <div className={styles.exampleCopy}>
            <span className={styles.eyebrow}>One query, multiple sources</span>
            <h2 id="example-title">Start with the catalogs you need.</h2>
            <p>Run your query. SuperSTAC combines the results across catalogs.</p>
            <div className={styles.diagram}><FederationMap /></div>
            <Link href="/docs/start/quickstart" className={styles.textLink}>Walk through a search <ArrowRight size={15} aria-hidden="true" /></Link>
          </div>
          <CodePreview />
        </section>

        <section className={styles.guides} aria-labelledby="guides-title">
          <div className={styles.sectionHeading}><h2 id="guides-title">Use it where you work.</h2><p>Pick your interface.</p></div>
          <div className={styles.guideGrid}>
            {guides.map(({ icon: Icon, ...guide }) => (
              <Link className={styles.guide} href={guide.href} key={guide.title}>
                <Icon size={20} aria-hidden="true" />
                <h3>{guide.title} <ArrowUpRight size={15} aria-hidden="true" /></h3>
                <p>{guide.text}</p>
              </Link>
            ))}
          </div>
        </section>

        <section className={styles.inventory} aria-labelledby="inventory-title">
          <div><h2 id="inventory-title">Same area. More questions.</h2><p>Save catalog metadata to GeoParquet and search it locally. Keep refining your study area without repeating every API request.</p></div>
          <div className={styles.inventoryLinks}>
            <Link href="/docs/guides/geoparquet" className={styles.textLink}>Work with saved metadata <ArrowRight size={15} aria-hidden="true" /></Link>
            <Link href="/docs/guides/benchmarks" className={styles.benchmark}>
              <strong>4.2 ms <span>local search</span></strong>
              <span>Selective query · 100,000 synthetic items</span>
              <span>Median of 5 runs on an M3 Pro. Excludes ingestion.</span>
              <span className={styles.benchmarkLink}>See benchmark and methodology <ArrowUpRight size={14} aria-hidden="true" /></span>
            </Link>
          </div>
        </section>
        <section className={styles.foundations} aria-labelledby="foundations-title">
          <div>
            <h2 id="foundations-title">Built on STAC. Powered by Rust.</h2>
            <p>Part of the open-source STAC ecosystem. SuperSTAC uses rustac’s <code>stac</code> types and <code>stac-io</code> for native catalog requests.</p>
          </div>
          <nav aria-label="Standards and open-source foundations">
            <a href="https://stacspec.org/">STAC <ArrowUpRight size={14} aria-hidden="true" /><span>The catalog standard</span></a>
            <a href="https://rust-lang.org/">Rust <ArrowUpRight size={14} aria-hidden="true" /><span>The search engine’s language</span></a>
            <a href="https://github.com/stac-utils/rustac">rustac <ArrowUpRight size={14} aria-hidden="true" /><span>STAC libraries for Rust</span></a>
          </nav>
        </section>

        <p className={styles.alpha}>SuperSTAC is in alpha. <Link href="/docs/reference/status">See current limitations.</Link></p>
      </div>
      <footer className={styles.footer}>
        <PoweredBySpatialnode />
        <nav aria-label="Project links"><a href="https://www.spatialnode.net/articles/introducing-superstac-many-catalogs-one-search2a5e11">Why SuperSTAC?</a><a href="https://github.com/spatialnode/superstac">GitHub</a><a href="https://github.com/spatialnode/superstac/blob/main/LICENSE">MIT license</a></nav>
      </footer>
    </HomeLayout>
  );
}
