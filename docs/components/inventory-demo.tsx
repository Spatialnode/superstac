'use client';

import { useState } from 'react';
import { Database, Download, Search, Pause, Play } from 'lucide-react';

export function InventoryDemo() {
  const [paused, setPaused] = useState(false);

  return (
    <div className="inventory-demo" data-paused={paused}>
      <div className="inventory-heading">
        <span>Save once. Search again.</span>
        <button type="button" className="diagram-animation-toggle" onClick={() => setPaused(!paused)}
          aria-label={paused ? 'Play inventory animation' : 'Pause inventory animation'}>
          {paused ? <Play size={13} /> : <Pause size={13} />}
        </button>
      </div>
      <div className="inventory-story" role="img" aria-label="Save metadata from live catalogs to a local GeoParquet inventory, then search different dates within its saved coverage without downloading the metadata again.">
        <div className="inventory-source"><Download size={18} /><span>Catalog metadata</span></div>
        <div className="inventory-transfer" aria-hidden="true"><span /><span /><span /></div>
        <div className="inventory-store"><Database size={24} /><div><strong>Your study area</strong><span>Saved to GeoParquet</span></div><span className="inventory-saved" aria-hidden="true">Saved</span></div>
        <div className="inventory-queries">
          {['Earlier dates', 'Later dates', 'A smaller area'].map((label, index) => (
            <div className={`inventory-query inventory-query-${index}`} key={label}>
              <Search size={14} /><span>{label}</span><span className="inventory-match">Local results</span>
            </div>
          ))}
        </div>
      </div>
      <p className="inventory-caption">Explore different queries within your saved coverage.</p>
    </div>
  );
}
