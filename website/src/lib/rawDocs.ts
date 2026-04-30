const docsPrefix = '@site/docs/';
const i18nDocsPattern =
  /^@site\/i18n\/([^/]+)\/docusaurus-plugin-content-docs\/current\/(.+)$/u;

export const rawDocsManifestUrl = '/raw/manifest.json';

export function getRawDocUrlFromSource(source: string | undefined): string | null {
  if (!source) {
    return null;
  }

  if (source.startsWith(docsPrefix)) {
    return `/raw/docs/${source.slice(docsPrefix.length)}`;
  }

  const match = source.match(i18nDocsPattern);
  if (match) {
    const [, locale, filePath] = match;
    return `/raw/i18n/${locale}/docs/${filePath}`;
  }

  return null;
}
