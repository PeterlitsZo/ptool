import React, {type ReactNode} from 'react';
import Head from '@docusaurus/Head';
import useBaseUrl from '@docusaurus/useBaseUrl';
import {PageMetadata} from '@docusaurus/theme-common';
import {useDoc} from '@docusaurus/plugin-content-docs/client';

import {getRawDocUrlFromSource, rawDocsManifestUrl} from '../../../lib/rawDocs';

export default function DocItemMetadata(): ReactNode {
  const {metadata, frontMatter, assets} = useDoc();
  const rawUrl = getRawDocUrlFromSource(metadata.source);
  const rawHref = useBaseUrl(rawUrl ?? '');
  const manifestHref = useBaseUrl(rawDocsManifestUrl);

  return (
    <>
      <PageMetadata
        title={metadata.title}
        description={metadata.description}
        keywords={frontMatter.keywords}
        image={assets.image ?? frontMatter.image}
      />
      {rawUrl && (
        <Head>
          <link
            rel="alternate"
            type="text/markdown"
            title={`${metadata.title} Markdown`}
            href={rawHref}
          />
          <link
            rel="alternate"
            type="application/json"
            title="ptool docs manifest"
            href={manifestHref}
          />
        </Head>
      )}
    </>
  );
}
