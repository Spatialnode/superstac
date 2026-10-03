'use client';
import { useEffect, useRef, useState } from 'react';

export function WasmPlayground() {
  const frame = useRef<HTMLIFrameElement>(null);
  const [height, setHeight] = useState(940);
  function sendTheme() {
    frame.current?.contentWindow?.postMessage({ type: 'superstac-theme', dark: document.documentElement.classList.contains('dark') }, location.origin);
  }
  useEffect(() => {
    function resize(event: MessageEvent) {
      if (event.origin !== location.origin || event.source !== frame.current?.contentWindow || event.data?.type !== 'superstac-height') return;
      if (Number.isFinite(event.data.height)) setHeight(Math.max(600, Math.min(2400, event.data.height + 4)));
    }
    window.addEventListener('message', resize);
    const observer = new MutationObserver(sendTheme);
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });
    return () => { window.removeEventListener('message', resize); observer.disconnect(); };
  }, []);
  return <iframe ref={frame} onLoad={sendTheme} src="/superstac/playground/index.html" title="Search satellite scenes with SuperSTAC" style={{ width: '100%', height, border: '1px solid var(--color-fd-border)', borderRadius: 12 }} />;
}
