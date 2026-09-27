import Link from 'next/link';
export default function NotFound() {
  return <main className="not-found"><p className="eyebrow">404 · OFF THE MAP</p><h1>This page isn’t here.</h1><p>Find your way back to the SuperSTAC documentation.</p><Link className="button-primary" href="/docs">Open documentation →</Link></main>;
}
