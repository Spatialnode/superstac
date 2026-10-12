import { $, el } from './dom.js';
import { codeFiles, codeTokens } from './code-examples.js';

export function codeView() {
  let config, query, files = [], source = '';
  function showFile() {
    const current = files[Number($('code-file').value) || 0];
    source = current?.content ?? '';
    $('code').replaceChildren(...codeTokens(source, current?.syntax).map(token => token.kind ? el('span', token.text, `token-${token.kind}`) : document.createTextNode(token.text)));
    $('copy').disabled = !current; $('copy-status').textContent = '';
    $('copy').textContent = current ? `Copy ${current.name}` : 'Copy code';
    $('code').scrollTop = 0;
  }
  function render() {
    if (!config || !query) return;
    const previous = files[Number($('code-file').value) || 0]?.name;
    try {
      const output = codeFiles(config, query, $('code-language').value); files = output.files;
      $('code-file').replaceChildren(...files.map((f, i) => new Option(f.name, String(i))));
      $('code-file').value = String(Math.max(0, files.findIndex(f => f.name === previous)));
      $('code-note').textContent = output.note; $('code-guide').href = output.guide;
      showFile();
    } catch (error) {
      files = []; source = ''; $('code-file').replaceChildren(); $('code').textContent = error.message;
      $('copy').disabled = true; $('copy').textContent = 'Copy code'; $('copy-status').textContent = ''; $('code-note').textContent = 'This query cannot be used with the selected interface.';
      $('code-guide').href = '/superstac/docs/reference/search';
    }
  }
  $('code-language').addEventListener('change', render);
  $('code-file').addEventListener('change', showFile);
  $('copy').addEventListener('click', async () => {
    if (!source) return;
    try { await navigator.clipboard.writeText(source); $('copy-status').textContent = 'Copied.'; }
    catch {
      const range = document.createRange(); range.selectNodeContents($('code'));
      const selection = window.getSelection(); selection.removeAllRanges(); selection.addRange(range);
      $('code').focus(); $('copy-status').textContent = 'Code selected. Press Ctrl+C or ⌘C to copy.';
    }
  });
  return {
    update(nextConfig, nextQuery) { config = nextConfig; query = nextQuery; render(); },
    error(message) { config = undefined; query = undefined; files = []; source = ''; $('code-file').replaceChildren(); $('code').textContent = message; $('copy').disabled = true; $('copy').textContent = 'Copy code'; $('copy-status').textContent = ''; $('code-note').textContent = 'Check your search fields to generate an example.'; $('code-guide').href = '/superstac/docs/reference/search'; },
  };
}
