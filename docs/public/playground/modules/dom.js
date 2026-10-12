export const $ = id => document.getElementById(id);
export function el(tag, text, className) {
  const node = document.createElement(tag);
  if (text !== undefined) node.textContent = text;
  if (className) node.className = className;
  return node;
}
export function button(label, action, className = 'secondary') {
  const node = el('button', label, className); node.type = 'button'; node.addEventListener('click', action); return node;
}
export function link(label, href) {
  const node = el('a', label, 'external-link'); node.href = href; node.target = '_blank'; node.rel = 'noopener noreferrer'; return node;
}
export function errorMessage(error) { return error instanceof Error ? error.message : String(error); }
