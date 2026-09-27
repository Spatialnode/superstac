'use client';
import { useState } from 'react';
import { Check, Copy } from 'lucide-react';
const examples = {
  Python: 'from superstac import Client\n\nclient = Client(catalogs=[\n    {"id": "earth-search", "url": "https://earth-search.aws.element84.com/v1"},\n    {"id": "microsoft", "url": "https://planetarycomputer.microsoft.com/api/stac/v1"},\n])\ntry:\n    result = client.search(collections=["sentinel-2-l2a"], limit=5)\n    for item in result.items():\n        print(item["id"])\nfinally:\n    client.shutdown()',
  Rust: 'use superstac_config::init_from_yaml;\nuse superstac_core::models::storage::Storage;\nuse superstac_engine::SuperSTACEngine;\nuse superstac_search::query::SearchQuery;\n\n#[tokio::main]\nasync fn main() -> Result<(), Box<dyn std::error::Error>> {\n    let engine = SuperSTACEngine::new(init_from_yaml(Storage::Memory, "superstac.yml")?);\n    let result = engine.search(SearchQuery {\n        collections: vec!["sentinel-2-l2a".into()], limit: Some(5),\n        ids: None, bbox: None, intersects: None, datetime: None, sortby: None,\n    }).await;\n    engine.shutdown().await;\n    println!("{} items", result?.metadata.total_items);\n    Ok(())\n}',
  CLI: '# Save the quickstart configuration as superstac.yml first.\nsuperstac collections\n\nsuperstac search \\\n  -c sentinel-2-l2a \\\n  -b 6.0,49.0,7.0,50.0 \\\n  -l 5\n\nsuperstac --json search -c sentinel-2-l2a -l 5',
};
export function CodePreview() {
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
    <pre id="example-panel" role="tabpanel" aria-labelledby={`tab-${tab}`} tabIndex={0}><code>{examples[tab]}</code></pre>
    <div className="code-footer"><span aria-live="polite">{copyFailed ? 'Select the code to copy it manually.' : copied ? 'Copied to clipboard.' : 'Real catalogs. Familiar interfaces.'}</span><span>superstac</span></div>
  </div>;
}
