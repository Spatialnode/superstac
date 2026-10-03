'use client';
import { useState } from 'react';
import { Check, Copy } from 'lucide-react';
import type { ReactNode } from 'react';
import { examples } from '@/lib/code-examples';
export function CodePreviewTabs({ highlighted }: { highlighted: Record<keyof typeof examples, ReactNode> }) {
  const [tab, setTab] = useState<keyof typeof examples>('Python');
  const [copied, setCopied] = useState(false);
  const [copyFailed, setCopyFailed] = useState(false);
  async function copy() {
    try { await navigator.clipboard.writeText(examples[tab]); setCopied(true); setCopyFailed(false); setTimeout(() => setCopied(false), 1600); }
    catch { setCopyFailed(true); }
  }
  return <div className="code-preview">
    <div className="code-toolbar"><div role="tablist" aria-label="Example language">{(Object.keys(examples) as (keyof typeof examples)[]).map((name) => <button key={name} id={`tab-${name}`} role="tab" aria-selected={tab === name} aria-controls="example-panel" tabIndex={tab === name ? 0 : -1} onKeyDown={(event) => {
      const names = Object.keys(examples) as (keyof typeof examples)[];
      const index = names.indexOf(name);
      const next = event.key === 'ArrowRight' ? (index + 1) % names.length : event.key === 'ArrowLeft' ? (index + names.length - 1) % names.length : event.key === 'Home' ? 0 : event.key === 'End' ? names.length - 1 : -1;
      if (next >= 0) { event.preventDefault(); setTab(names[next]); setCopied(false); setCopyFailed(false); document.getElementById(`tab-${names[next]}`)?.focus(); }
    }} onClick={() => { setTab(name); setCopied(false); setCopyFailed(false); }}>{name}</button>)}</div><button onClick={copy} aria-label={copied ? 'Copied' : 'Copy example'}>{copied ? <Check size={15}/> : <Copy size={15}/>}</button></div>
    <div id="example-panel" role="tabpanel" aria-labelledby={`tab-${tab}`} tabIndex={0}>{highlighted[tab]}</div>
    <div className="code-footer"><span aria-live="polite">{copyFailed ? 'Select the code to copy it manually.' : copied ? 'Copied to clipboard.' : 'See the quickstart for setup.'}</span><span>superstac</span></div>
  </div>;
}
