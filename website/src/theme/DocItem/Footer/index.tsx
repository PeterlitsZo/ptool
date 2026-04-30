import React, {type ReactNode} from 'react';
import clsx from 'clsx';
import Translate from '@docusaurus/Translate';
import useBaseUrl from '@docusaurus/useBaseUrl';
import {ThemeClassNames} from '@docusaurus/theme-common';
import {useDoc} from '@docusaurus/plugin-content-docs/client';
import TagsListInline from '@theme/TagsListInline';
import EditMetaRow from '@theme/EditMetaRow';

import {getRawDocUrlFromSource, rawDocsManifestUrl} from '../../../lib/rawDocs';

export default function DocItemFooter(): ReactNode {
  const {metadata} = useDoc();
  const {editUrl, lastUpdatedAt, lastUpdatedBy, source, tags} = metadata;
  const rawUrl = getRawDocUrlFromSource(source);
  const rawHref = useBaseUrl(rawUrl ?? '');
  const manifestHref = useBaseUrl(rawDocsManifestUrl);

  const canDisplayTagsRow = tags.length > 0;
  const canDisplayAiRow = !!rawUrl;
  const canDisplayEditMetaRow = !!(editUrl || lastUpdatedAt || lastUpdatedBy);

  const canDisplayFooter =
    canDisplayTagsRow || canDisplayAiRow || canDisplayEditMetaRow;

  if (!canDisplayFooter) {
    return null;
  }

  return (
    <footer
      className={clsx(ThemeClassNames.docs.docFooter, 'docusaurus-mt-lg')}>
      {canDisplayTagsRow && (
        <div
          className={clsx(
            'row margin-top--sm',
            ThemeClassNames.docs.docFooterTagsRow,
          )}>
          <div className="col">
            <TagsListInline tags={tags} />
          </div>
        </div>
      )}
      {canDisplayAiRow && (
        <div
          className={clsx(
            'margin-top--sm padding--md',
            'theme-admonition theme-admonition-note alert alert--secondary',
          )}>
          <p className="margin-bottom--sm">
            <strong>
              <Translate id="docItemFooter.aiAccess.title">AI access:</Translate>
            </strong>{' '}
            <Translate id="docItemFooter.aiAccess.body">
              Give your assistant the raw Markdown URL for this page, or start
              from the docs manifest.
            </Translate>
          </p>
          <div className="margin-bottom--none">
            <a href={rawHref}>
              <Translate id="docItemFooter.aiAccess.rawLink">
                Raw Markdown
              </Translate>
            </a>
            {' · '}
            <a href={manifestHref}>
              <Translate id="docItemFooter.aiAccess.manifestLink">
                Docs Manifest
              </Translate>
            </a>
          </div>
        </div>
      )}
      {canDisplayEditMetaRow && (
        <EditMetaRow
          className={clsx(
            'margin-top--sm',
            ThemeClassNames.docs.docFooterEditMetaRow,
          )}
          editUrl={editUrl}
          lastUpdatedAt={lastUpdatedAt}
          lastUpdatedBy={lastUpdatedBy}
        />
      )}
    </footer>
  );
}
