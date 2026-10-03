import { VersionedDocsLayout } from '@/components/docs-layout';
import { source } from '@/lib/source';

export default function Layout({ children }: { children: React.ReactNode }) {
  return <VersionedDocsLayout tree={source.getPageTree()}>{children}</VersionedDocsLayout>;
}
