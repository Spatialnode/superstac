import Link from 'next/link';
import { HomeLayout } from 'fumadocs-ui/layouts/home';
import { ArrowRight, ArrowUpRight, Braces, Layers, Terminal, GitMerge, Scan, BookOpen } from 'lucide-react';
import { baseOptions } from '@/lib/layout.shared';
import { FederationMap } from '@/components/federation-map';
import { CodePreview } from '@/components/code-preview';
import { PoweredBySpatialnode } from '@/components/powered-by-spatialnode';
import type { Metadata } from 'next';

export const metadata: Metadata = { alternates: { canonical: '/superstac' } };
const paths = [
  { icon: Braces, label: 'Python', title: 'From notebook to pipeline.', text: 'A familiar client, ordinary dictionaries, and sync or async search.', href: '/docs/python/overview' },
  { icon: Layers, label: 'Rust', title: 'Built into your application.', text: 'Embed the engine. Keep control of configuration and lifecycle.', href: '/docs/rust/overview' },
  { icon: Terminal, label: 'Command line', title: 'An answer from your terminal.', text: 'Discover collections, search an area, and inspect the results.', href: '/docs/cli/overview' },
];
export default function Home() {
  return <HomeLayout {...baseOptions()}>
    <div className="landing" id="main-content">
      <section className="landing-hero">
        <div className="hero-copy">
          <Link href="/docs/reference/status" className="release-pill"><span className="tiny-dot"/> OPEN SOURCE · V0.1 ALPHA <ArrowUpRight size={12}/></Link>
          <h1>Many catalogs.<br/><span>One search.</span></h1>
          <p className="hero-description">Search across STAC catalogs through one interface, with Python, Rust, or the command line.</p>
          <div className="hero-actions"><Link href="/docs/start/quickstart" className="button-primary">Start searching <ArrowRight size={16}/></Link><Link href="/docs" className="button-secondary"><BookOpen size={16}/> Explore the docs</Link></div>
          <div className="hero-install"><span aria-hidden="true">$</span><code>pip install superstac</code><span className="install-note">Python · Rust · CLI</span></div>
        </div>
        <FederationMap/>
      </section>
      <div className="principles-strip"><span>A <a href="https://spatialnode.com">Spatialnode ↗</a> project</span><span>STAC native</span><span>Rust engine</span><span>MIT licensed</span></div>
      <section className="start-section" aria-labelledby="choose-title"><div className="section-intro"><p className="eyebrow">MEET YOUR NEXT WORKFLOW</p><h2 id="choose-title">Start where you work.</h2><p>One engine. Three ways to make it yours.</p></div><div className="path-cards">{paths.map(({icon:Icon,...path},i) => <Link className="path-card" href={path.href} key={path.label}><div className="path-card-top"><Icon size={23}/><span>0{i+1}</span></div><span className="path-label">{path.label}</span><h3>{path.title}</h3><p>{path.text}</p><span className="path-link">Read the guide <ArrowUpRight size={15}/></span></Link>)}</div></section>
      <section className="code-section"><div className="code-section-copy"><p className="eyebrow">LESS PLUMBING. MORE DISCOVERY.</p><h2>Different sources.<br/>A shared vocabulary.</h2><p>Register your catalogs, choose an area and a time range, and get back STAC items. SuperSTAC handles the search across sources.</p><ul><li><Scan size={18}/><span>Query the catalogs that can serve your collections.</span></li><li><GitMerge size={18}/><span>Normalize names and remove duplicate item IDs.</span></li><li><Layers size={18}/><span>Inspect failures and counts with each response.</span></li></ul><Link href="/docs/concepts/federation" className="text-link">How federation works <ArrowRight size={15}/></Link></div><CodePreview/></section>
      <section className="reference-banner"><div><p className="eyebrow">WHEN YOU NEED THE DETAILS</p><h2>Make the next query your own.</h2></div><div><Link href="/docs/reference/configuration">Configuration <ArrowUpRight size={16}/></Link><Link href="/docs/python/api">Python API <ArrowUpRight size={16}/></Link><Link href="/docs/reference/troubleshooting">Troubleshooting <ArrowUpRight size={16}/></Link></div></section>
      <div className="alpha-note"><span className="tiny-dot"/><p>Built in the open. SuperSTAC is in alpha; APIs and configuration may change. <Link href="/docs/reference/status">See current capabilities →</Link></p></div>
    </div>
    <footer className="site-footer"><PoweredBySpatialnode/><div><a href="https://github.com/spatialnode/superstac">GitHub ↗</a><a href="https://github.com/spatialnode/superstac/blob/main/LICENSE">MIT license ↗</a></div></footer>
  </HomeLayout>;
}
