import Link from 'next/link';
import { HomeLayout } from 'fumadocs-ui/layouts/home';
import { ArrowRight, ArrowUpRight, Braces, Layers, Terminal, GitMerge, Scan, BookOpen } from 'lucide-react';
import { baseOptions } from '@/lib/layout.shared';
import { FederationMap } from '@/components/federation-map';
import { CodePreview } from '@/components/code-preview';
import { InventoryDemo } from '@/components/inventory-demo';
import { PoweredBySpatialnode } from '@/components/powered-by-spatialnode';
import type { Metadata } from 'next';

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
      <div className="landing" id="main-content">
        <section className="landing-hero">
          <div className="hero-copy">
            <Link href="/docs/releases" className="release-pill">
              <span className="tiny-dot" /> v0.3 · What’s new <ArrowUpRight size={12} />
            </Link>
            <h1>Many catalogs.<br /><span>One search.</span></h1>
            <p className="hero-description">Find satellite imagery across providers with one query. Combine the results, save the metadata you need, and explore it again with fewer API requests.</p>
            <div className="hero-actions">
              <a href="/superstac/playground" className="button-primary">Open playground <ArrowRight size={16} /></a>
              <Link href="/docs/start/installation" className="button-secondary"><BookOpen size={16} /> Installation</Link>
            </div>
            <div className="hero-install"><span aria-hidden="true">$</span><code>pip install superstac</code></div>
          </div>
          <FederationMap />
        </section>

        <section className="start-section" aria-labelledby="benefits-title">
          <div className="section-intro">
            <h2 id="benefits-title">Find imagery. Keep the metadata you need.</h2>
            <p>Search from a browser app, Python notebook, Rust application, or the command line.</p>
          </div>
          <div className="path-cards">
            <Link className="path-card" href="/docs/concepts/federation">
              <Scan className="path-icon" size={23} />
              <h3>Search across providers</h3>
              <p>Query relevant catalogs concurrently and receive combined results, with visibility into which sources responded or failed.</p>
              <span className="path-link">How federation works <ArrowRight size={15} /></span>
            </Link>
            <Link className="path-card" href="/docs/guides/geoparquet">
              <Layers className="path-icon" size={23} />
              <h3>Save once. Explore repeatedly.</h3>
              <p>Save your study-area metadata to GeoParquet and reuse it across searches. Automatic mode checks live catalogs when the saved data is too old or doesn’t cover your query.</p>
              <span className="path-link">Build your inventory <ArrowRight size={15} /></span>
            </Link>
            <Link className="path-card" href="/docs/start/quickstart">
              <GitMerge className="path-icon" size={23} />
              <h3>Spend less time combining results</h3>
              <p>Configure collection and asset aliases, remove duplicate item IDs, and pass STAC metadata into your existing analysis workflow.</p>
              <span className="path-link">Run your first search <ArrowRight size={15} /></span>
            </Link>
          </div>
        </section>

        <section className="code-section" aria-labelledby="inventory-title">
          <div className="code-section-copy">
            <h2 id="inventory-title">Keep exploring the same study area.</h2>
            <p>Exploring the same region over multiple sessions? Save its catalog metadata to GeoParquet, then refine your searches locally. Scope ingestion by collection, area, and time so you only collect what you need.</p>
            <ul>
              <li><Layers size={18} /><span>Follow ingestion progress, set a storage limit, and resume interrupted work.</span></li>
              <li><Scan size={18} /><span>Search the saved coverage locally; let automatic mode fall back to live catalogs when needed.</span></li>
              <li><GitMerge size={18} /><span>Add recent dates and clean up old files as your project grows.</span></li>
            </ul>
            <p>Inventories store metadata. Imagery previews and downloads still use the provider’s assets.</p>
            <Link href="/docs/python/geoparquet-notebook" className="text-link">Try the GeoParquet notebook <ArrowRight size={15} /></Link>
          </div>
          <div className="code-section-copy">
            <InventoryDemo />
            <h3 className="benchmark-heading">How fast is local search?</h3>
            <p>Our benchmark measures how skipping irrelevant data speeds up a selective search over 100,000 synthetic items. See the results and run it yourself.</p>
            <Link href="/docs/guides/benchmarks" className="text-link">Explore the benchmark <ArrowRight size={15} /></Link>
          </div>
        </section>

        <section className="start-section" aria-labelledby="choose-title">
          <div className="section-intro"><h2 id="choose-title">Guides</h2></div>
          <div className="path-cards interface-cards">
            {guides.map(({ icon: Icon, ...guide }) => (
              <Link className="path-card" href={guide.href} key={guide.title}>
                <Icon className="path-icon" size={23} />
                <h3>{guide.title}</h3>
                <p>{guide.text}</p>
                <span className="path-link">Read the guide <ArrowRight size={15} /></span>
              </Link>
            ))}
          </div>
        </section>

        <section className="code-section" aria-labelledby="example-title">
          <div className="code-section-copy">
            <h2 id="example-title">Search example</h2>
            <p>Register the catalogs you want to search, then query a collection. SuperSTAC sends the request to the relevant catalogs and combines their results.</p>
            <ul>
              <li><Scan size={18} /><span>Filter by location and date.</span></li>
              <li><GitMerge size={18} /><span>Normalize collection and asset names, and remove duplicate item IDs.</span></li>
              <li><Layers size={18} /><span>Check which catalogs responded and which failed.</span></li>
            </ul>
            <Link href="/docs/start/quickstart" className="text-link">Full example and configuration <ArrowRight size={15} /></Link>
          </div>
          <CodePreview />
        </section>

        <section className="article-section" aria-labelledby="article-title">
          <div className="section-intro">
            <h2 id="article-title">Why SuperSTAC?</h2>
            <p>The pipeline problem that led to SuperSTAC, and the thinking behind it.</p>
          </div>
          <a href="https://www.spatialnode.net/articles/introducing-superstac-many-catalogs-one-search2a5e11" className="text-link">
            Read the story on Spatialnode <ArrowUpRight size={15} aria-hidden="true" />
          </a>
        </section>

        <section className="reference-banner" aria-labelledby="reference-title">
          <h2 id="reference-title">Reference</h2>
          <div>
            <Link href="/docs/reference/configuration">Configuration <ArrowUpRight size={16} /></Link>
            <Link href="/docs/python/api">Python API <ArrowUpRight size={16} /></Link>
            <Link href="/docs/reference/troubleshooting">Troubleshooting <ArrowUpRight size={16} /></Link>
          </div>
        </section>
        <div className="alpha-note">
          <span className="tiny-dot" />
          <p>SuperSTAC is in alpha. APIs and configuration may change. <Link href="/docs/reference/status">Current limitations →</Link></p>
        </div>
      </div>
      <footer className="site-footer">
        <PoweredBySpatialnode />
        <div><a href="https://github.com/spatialnode/superstac">GitHub ↗</a><a href="https://github.com/spatialnode/superstac/blob/main/LICENSE">MIT license ↗</a></div>
      </footer>
    </HomeLayout>
  );
}
