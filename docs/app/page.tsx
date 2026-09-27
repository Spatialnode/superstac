import Link from 'next/link';
import { HomeLayout } from 'fumadocs-ui/layouts/home';
import { ArrowRight, ArrowUpRight, Braces, Layers, Terminal, GitMerge, Scan, BookOpen } from 'lucide-react';
import { baseOptions } from '@/lib/layout.shared';
import { FederationMap } from '@/components/federation-map';
import { CodePreview } from '@/components/code-preview';
import { PoweredBySpatialnode } from '@/components/powered-by-spatialnode';
import type { Metadata } from 'next';

export const metadata: Metadata = { alternates: { canonical: '/superstac' } };

const guides = [
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
            <Link href="/docs/reference/status" className="release-pill">
              <span className="tiny-dot" /> v0.1 · Alpha <ArrowUpRight size={12} />
            </Link>
            <h1>Many catalogs.<br /><span>One search.</span></h1>
            <p className="hero-description">Search across STAC catalogs through one interface, with Python, Rust, or the command line.</p>
            <div className="hero-actions">
              <Link href="/docs/start/quickstart" className="button-primary">Quickstart <ArrowRight size={16} /></Link>
              <Link href="/docs/start/installation" className="button-secondary"><BookOpen size={16} /> Installation</Link>
            </div>
            <div className="hero-install"><span aria-hidden="true">$</span><code>pip install superstac</code></div>
          </div>
          <FederationMap />
        </section>

        <section className="start-section" aria-labelledby="choose-title">
          <div className="section-intro"><h2 id="choose-title">Guides</h2></div>
          <div className="path-cards">
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
