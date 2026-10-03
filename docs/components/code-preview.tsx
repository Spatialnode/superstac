import { highlight } from 'fumadocs-core/highlight';
import { examples } from '@/lib/code-examples';
import { CodePreviewTabs } from './code-preview-tabs';

export async function CodePreview() {
  const [Python, JavaScript, Rust, CLI] = await Promise.all([
    highlight(examples.Python, { lang: 'python', theme: 'github-dark' }),
    highlight(examples.JavaScript, { lang: 'javascript', theme: 'github-dark' }),
    highlight(examples.Rust, { lang: 'rust', theme: 'github-dark' }),
    highlight(examples.CLI, { lang: 'bash', theme: 'github-dark' }),
  ]);

  return <CodePreviewTabs highlighted={{ Python, JavaScript, Rust, CLI }} />;
}
