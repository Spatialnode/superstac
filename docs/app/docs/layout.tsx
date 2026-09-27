import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { source } from '@/lib/source';
import { baseOptions } from '@/lib/layout.shared';

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <DocsLayout {...baseOptions()} tree={source.getPageTree()} sidebar={{ banner: <div className="sidebar-version"><span className="tiny-dot" /> v0.2 · Alpha</div> }}>
      {children}
    </DocsLayout>
  );
}
