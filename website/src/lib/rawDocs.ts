const docsPrefix = '@site/docs/';
const versionedDocsPattern = /^@site\/versioned_docs\/version-([^/]+)\/(.+)$/u;
const i18nDocsPattern =
  /^@site\/i18n\/([^/]+)\/docusaurus-plugin-content-docs\/current\/(.+)$/u;
const i18nVersionedDocsPattern =
  /^@site\/i18n\/([^/]+)\/docusaurus-plugin-content-docs\/version-([^/]+)\/(.+)$/u;

export const rawDocsManifestUrl = '/raw/manifest.json';

export function getRawDocUrlFromSource(source: string | undefined): string | null {
  if (!source) {
    return null;
  }

  if (source.startsWith(docsPrefix)) {
    return `/raw/docs/${source.slice(docsPrefix.length)}`;
  }

  const versionedMatch = source.match(versionedDocsPattern);
  if (versionedMatch) {
    const [, versionName, filePath] = versionedMatch;
    return `/raw/versioned_docs/version-${versionName}/${filePath}`;
  }

  const match = source.match(i18nDocsPattern);
  if (match) {
    const [, locale, filePath] = match;
    return `/raw/i18n/${locale}/docs/${filePath}`;
  }

  const i18nVersionedMatch = source.match(i18nVersionedDocsPattern);
  if (i18nVersionedMatch) {
    const [, locale, versionName, filePath] = i18nVersionedMatch;
    return `/raw/i18n/${locale}/versioned_docs/version-${versionName}/${filePath}`;
  }

  return null;
}
