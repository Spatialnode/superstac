import { Layers, ArrowUpRight, Check } from 'lucide-react';
export function FederationMap() {
  return (
    <div className="federation-map" role="img" aria-label="One SuperSTAC query branches to Earth Search, Planetary Computer, and your STAC API, then returns a combined result.">
      <div className="map-grid" aria-hidden="true" />
      <svg className="contours" viewBox="0 0 600 450" aria-hidden="true"><g fill="none" stroke="currentColor" strokeWidth="1"><path d="M-80 180Q60-70 170 50T420 40T680 90M-70 210Q70-40 180 80T430 70T690 120M-60 240Q80-10 190 110T440 100T700 150M-50 270Q90 20 200 140T450 130T710 180M-40 300Q100 50 210 170T460 160T720 210M-30 330Q110 80 220 200T470 190T730 240"/></g></svg>
      <div className="map-label"><span>01 / FEDERATED SEARCH</span><span className="map-cross">+</span></div>
      <svg className="network-lines" viewBox="0 0 600 450" aria-hidden="true"><g fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M157 214H210Q240 214 240 123H330"/><path d="M157 214H330"/><path d="M157 214H210Q240 214 240 305H330"/></g><g fill="currentColor"><circle cx="330" cy="123" r="3"/><circle cx="330" cy="214" r="3"/><circle cx="330" cy="305" r="3"/></g></svg>
      <div className="query-node"><span>YOUR QUERY</span><code>search()</code><ArrowUpRight size={15} /></div>
      <div className="catalog-node catalog-one"><Layers size={20}/><div><strong>Earth Search</strong><small>Element 84</small></div><span className="tiny-dot"/></div>
      <div className="catalog-node catalog-two"><Layers size={20}/><div><strong>Planetary Computer</strong><small>Microsoft</small></div><span className="tiny-dot"/></div>
      <div className="catalog-node catalog-three"><Layers size={20}/><div><strong>Your catalog</strong><small>Compatible STAC API</small></div><span className="tiny-dot"/></div>
      <div className="map-result"><Check size={13}/><span>Combined items. Consistent names.</span></div>
      <div className="map-coordinate">41°23′ N &nbsp; 2°10′ E</div>
    </div>
  );
}
