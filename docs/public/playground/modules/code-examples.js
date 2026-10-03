import { example } from './model.js';

function python(value, depth = 0) {
  const indent = '    '.repeat(depth), next = indent + '    ';
  if (value === null) return 'None';
  if (typeof value === 'boolean') return value ? 'True' : 'False';
  if (Array.isArray(value)) return value.length ? `[\n${value.map(v => next + python(v, depth + 1)).join(',\n')},\n${indent}]` : '[]';
  if (typeof value === 'object') {
    const pairs = Object.entries(value);
    return pairs.length ? `{\n${pairs.map(([k, v]) => `${next}${JSON.stringify(k)}: ${python(v, depth + 1)}`).join(',\n')},\n${indent}}` : '{}';
  }
  return JSON.stringify(value);
}
function yaml(value, depth = 0) {
  const indent = '  '.repeat(depth);
  const scalar = v => v === null || typeof v !== 'object' || Object.keys(v).length === 0;
  if (Array.isArray(value)) return value.map(v => scalar(v) ? `${indent}- ${JSON.stringify(v)}` : `${indent}-\n${yaml(v, depth + 1)}`).join('\n');
  return Object.entries(value).map(([k, v]) => scalar(v) ? `${indent}${JSON.stringify(k)}: ${JSON.stringify(v)}` : `${indent}${JSON.stringify(k)}:\n${yaml(v, depth + 1)}`).join('\n');
}
const shellQuote = value => `'${String(value).replaceAll("'", "'\\''")}'`;
function rustString(value) {
  let hashes = '#';
  while (value.includes('"' + hashes)) hashes += '#';
  return `r${hashes}"${value}"${hashes}`;
}
export function codeFiles(config, query, language) {
  const file = (name, syntax, content) => ({ name, syntax, content });
  if (language === 'JavaScript') return { note: 'Run in your browser app after installing the WASM package.', guide: '/superstac/docs/wasm/overview', files: [file('search.js', 'javascript', example(config, query))] };
  if (language === 'Python') {
    const args = Object.entries(query).map(([key, value]) => `        ${key}=${python(value, 2)},`).join('\n');
    return { note: 'Install superstac, then run python search.py or uv run search.py.', guide: '/superstac/docs/start/installation', files: [file('search.py', 'python', `from superstac import Client\n\nconfig = ${python(config)}\nclient = Client(config)\ntry:\n    result = client.search(\n${args}\n    )\n    for item in result.items():\n        print(item["id"], item["properties"].get("datetime"))\n    for failure in result.metadata["failures"]:\n        print(failure["catalog_id"], failure["reason"])\nfinally:\n    client.shutdown()\n`)] };
  }
  const native = { providers: [], catalogs: config.catalogs, settings: {
    health_check_strategy: 'hourly', healthy_status_code_range: [200, 299], auto_fix_duplicate_catalog_id: true,
    auto_fix_duplicate_provider_id: true, log_level: 'info', logging_enabled: true, ...config.settings,
  } };
  const configFile = file('superstac.yml', 'yaml', yaml(native) + '\n');
  if (language === 'CLI') {
    if (query.intersects) throw new Error('The CLI does not support GeoJSON geometry filters. Use Python, JavaScript, or Rust, or choose a bounding box in Search.');
    const args = ['superstac --json search'];
    for (const collection of query.collections ?? []) args.push(`--collection=${shellQuote(collection)}`);
    for (const id of query.ids ?? []) args.push(`--id=${shellQuote(id)}`);
    if (query.bbox) args.push(`--bbox=${shellQuote(query.bbox.join(','))}`);
    if (query.datetime) args.push(`--datetime=${shellQuote(query.datetime)}`);
    for (const sort of query.sortby ?? []) args.push(`--sortby=${shellQuote(sort)}`);
    args.push(`--limit=${shellQuote(query.limit ?? 10)}`);
    return { note: 'Save both files in one folder. Install the CLI, then run sh search.sh.', guide: '/superstac/docs/cli/overview', files: [file('search.sh', 'bash', `# Uses superstac.yml in the current directory.\n${args.join(' \\\n  ')}\n`), configFile] };
  }
  if (language !== 'Rust') throw new Error('Choose a supported language.');
  return { note: 'Save all three files with the paths shown, then run cargo run.', guide: '/superstac/docs/rust/overview', files: [
    file('src/main.rs', 'rust', `use superstac_config::init_from_yaml;\nuse superstac_core::models::storage::Storage;\nuse superstac_engine::SuperSTACEngine;\nuse superstac_search::query::SearchQuery;\n\n#[tokio::main]\nasync fn main() -> Result<(), Box<dyn std::error::Error>> {\n    let query: SearchQuery = serde_json::from_str(\n        ${rustString(JSON.stringify(query, null, 2))},\n    )?;\n    let storage = init_from_yaml(Storage::Memory, "superstac.yml")?;\n    let engine = SuperSTACEngine::new(storage);\n    let result = engine.search(query).await;\n    engine.shutdown().await;\n    let result = result?;\n    for entry in result.items {\n        println!("{} from {:?}", entry.item.id, entry.seen_in);\n    }\n    for failure in result.metadata.failures {\n        eprintln!("{}: {}", failure.catalog_id, failure.reason);\n    }\n    Ok(())\n}\n`),
    file('Cargo.toml', 'toml', `[package]\nname = "superstac-example"\nversion = "0.1.0"\nedition = "2021"\nrust-version = "1.88"\n\n[dependencies]\nsuperstac-core = "0.3"\nsuperstac-config = "0.3"\nsuperstac-search = "0.3"\nsuperstac-engine = "0.3"\nserde_json = "1"\ntokio = { version = "1", features = ["macros", "rt-multi-thread"] }\n`), configFile,
  ] };
}

// Tokenize only for display. Copying always uses the original source string.
export function codeTokens(source, language) {
  const hashComments = ['python', 'bash', 'yaml', 'toml'].includes(language);
  const pattern = hashComments
    ? /#[^\n]*|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|\b(?:from|import|try|finally|for|in|if|print|True|False|None)\b|\b\d+(?:\.\d+)?\b/g
    : /\/\/[^\n]*|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|\b(?:import|from|const|let|await|async|try|finally|for|of|if|else|use|fn|pub|mut|match|true|false|null|Some|None|Ok)\b|\b\d+(?:\.\d+)?\b/g;
  const tokens = []; let end = 0;
  for (const match of source.matchAll(pattern)) {
    if (match.index > end) tokens.push({ text: source.slice(end, match.index) });
    const text = match[0];
    const kind = text.startsWith('//') || (hashComments && text.startsWith('#')) ? 'comment' : /^["']/.test(text) ? 'string' : /^\d/.test(text) ? 'number' : 'keyword';
    tokens.push({ text, kind }); end = match.index + text.length;
  }
  if (end < source.length) tokens.push({ text: source.slice(end) });
  return tokens;
}
