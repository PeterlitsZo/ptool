import {cp, mkdir, readdir, readFile, rm, writeFile} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const websiteDir = path.resolve(__dirname, '..');
const docsDir = path.join(websiteDir, 'docs');
const versionedDocsDir = path.join(websiteDir, 'versioned_docs');
const i18nDir = path.join(websiteDir, 'i18n');
const outputDir = path.join(websiteDir, 'static', 'raw');

const locales = ['en', 'zh-Hans', 'es', 'pt-BR', 'ja'];
const currentVersionName = 'current';
const currentVersionPath = 'unreleased';

async function main() {
  await rm(outputDir, {recursive: true, force: true});
  await mkdir(outputDir, {recursive: true});

  const manifest = [];
  const stableVersions = await readVersionNames();
  const lastVersionName = stableVersions[0] ?? null;

  await exportTree({
    lastVersionName,
    locale: 'en',
    sourceDir: docsDir,
    outputSubdir: path.join('docs'),
    manifest,
    versionName: currentVersionName,
  });

  for (const versionName of stableVersions) {
    await exportTree({
      locale: 'en',
      sourceDir: path.join(versionedDocsDir, `version-${versionName}`),
      outputSubdir: path.join('versioned_docs', `version-${versionName}`),
      manifest,
      versionName,
    });
  }

  for (const locale of locales) {
    if (locale === 'en') {
      continue;
    }

    await exportTree({
      lastVersionName,
      locale,
      sourceDir: path.join(
        i18nDir,
        locale,
        'docusaurus-plugin-content-docs',
        'current',
      ),
      outputSubdir: path.join('i18n', locale, 'docs'),
      manifest,
      versionName: currentVersionName,
    });

    for (const versionName of stableVersions) {
      await exportTree({
        locale,
        sourceDir: path.join(
          i18nDir,
          locale,
          'docusaurus-plugin-content-docs',
          `version-${versionName}`,
        ),
        outputSubdir: path.join(
          'i18n',
          locale,
          'versioned_docs',
          `version-${versionName}`,
        ),
        manifest,
        versionName,
      });
    }
  }

  manifest.sort((a, b) => a.permalink.localeCompare(b.permalink));
  await writeFile(
    path.join(outputDir, 'manifest.json'),
    `${JSON.stringify({generatedAt: new Date().toISOString(), docs: manifest}, null, 2)}\n`,
    'utf8',
  );
}

async function exportTree({
  locale,
  lastVersionName,
  sourceDir,
  outputSubdir,
  manifest,
  versionName,
}) {
  const files = await collectMarkdownFiles(sourceDir);

  for (const file of files) {
    const relativePath = path.relative(sourceDir, file);
    const outputPath = path.join(outputDir, outputSubdir, relativePath);
    await mkdir(path.dirname(outputPath), {recursive: true});
    await cp(file, outputPath);

    manifest.push(
      await createManifestEntry({
        locale,
        lastVersionName,
        sourceDir,
        outputSubdir,
        file,
        versionName,
      }),
    );
  }
}

async function readVersionNames() {
  try {
    const content = await readFile(path.join(websiteDir, 'versions.json'), 'utf8');
    const parsed = JSON.parse(content);
    return Array.isArray(parsed) ? parsed : [];
  } catch (error) {
    if (error && typeof error === 'object' && 'code' in error && error.code === 'ENOENT') {
      return [];
    }
    throw error;
  }
}

async function collectMarkdownFiles(dir) {
  const entries = await readdir(dir, {withFileTypes: true});
  const files = await Promise.all(
    entries.map(async (entry) => {
      const entryPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        return collectMarkdownFiles(entryPath);
      }
      if (entry.isFile() && /\.(md|mdx)$/u.test(entry.name)) {
        return [entryPath];
      }
      return [];
    }),
  );

  return files.flat().sort((a, b) => a.localeCompare(b));
}

async function createManifestEntry({
  locale,
  lastVersionName,
  sourceDir,
  outputSubdir,
  file,
  versionName,
}) {
  const relativePath = path.relative(sourceDir, file);
  const rawUrl = `/${toPosixPath(path.join('raw', outputSubdir, relativePath))}`;
  const sourcePath = toPosixPath(path.relative(websiteDir, file));
  const docPath = normalizeDocPath(relativePath);

  return {
    title: await extractTitle(file),
    locale,
    sourcePath,
    rawUrl,
    version: versionName,
    permalink: toPermalink({locale, docPath, lastVersionName, versionName}),
  };
}

function normalizeDocPath(relativePath) {
  const withoutExtension = relativePath.replace(/\.(md|mdx)$/u, '');
  const normalized = toPosixPath(withoutExtension);
  if (normalized === 'index') {
    return '';
  }
  if (normalized.endsWith('/index')) {
    return normalized.slice(0, -'/index'.length);
  }
  return normalized;
}

function toPermalink({locale, docPath, lastVersionName, versionName}) {
  const localePrefix = locale === 'en' ? '' : `/${locale}`;
  const docPrefix = '/docs';

  let basePath = `${localePrefix}${docPrefix}`;
  if (versionName === currentVersionName) {
    basePath = `${basePath}/${currentVersionPath}`;
  } else if (versionName !== lastVersionName) {
    basePath = `${basePath}/${versionName}`;
  }

  if (!docPath) {
    return basePath;
  }
  return `${basePath}/${docPath}`;
}

async function extractTitle(file) {
  const content = await readFile(file, 'utf8');
  const frontmatterTitle = content.match(/^title:\s+(.+)$/mu)?.[1]?.trim();
  if (frontmatterTitle) {
    return stripQuotes(frontmatterTitle);
  }

  const headingTitle = content.match(/^#\s+(.+)$/mu)?.[1]?.trim();
  if (headingTitle) {
    return headingTitle;
  }

  return path.basename(file, path.extname(file));
}

function stripQuotes(value) {
  if (
    (value.startsWith('"') && value.endsWith('"')) ||
    (value.startsWith("'") && value.endsWith("'"))
  ) {
    return value.slice(1, -1);
  }
  return value;
}

function toPosixPath(filePath) {
  return filePath.split(path.sep).join('/');
}

await main();
