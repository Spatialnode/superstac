import defaultMdxComponents from 'fumadocs-ui/mdx';
import type { MDXComponents } from 'mdx/types';
import Link from 'next/link';
import type { ComponentProps } from 'react';

function ContentLink({ href = '', ...props }: ComponentProps<'a'>) {
  // Public downloads already include basePath; normal documentation links don't.
  if (href.startsWith('/superstac/') || !href.startsWith('/')) return <a href={href} {...props} />;
  return <Link href={href} {...props} />;
}

export function getMDXComponents(components?: MDXComponents): MDXComponents {
  return { ...defaultMdxComponents, a: ContentLink, ...components };
}
