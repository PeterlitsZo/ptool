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
const manifestOutputSubdir = 'manifests';

async function main() {
  await rm(outputDir, {recursive: true, force: true});
  await mkdir(outputDir, {recursive: true});

  const manifests = new Map();
  const stableVersions = await readVersionNames();
  const lastVersionName = stableVersions[0] ?? null;

  await exportTree({
    lastVersionName,
    locale: 'en',
    sourceDir: docsDir,
    outputSubdir: path.join('docs'),
    manifests,
    versionName: currentVersionName,
  });

  for (const versionName of stableVersions) {
    await exportTree({
      locale: 'en',
      sourceDir: path.join(versionedDocsDir, `version-${versionName}`),
      outputSubdir: path.join('versioned_docs', `version-${versionName}`),
      manifests,
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
      manifests,
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
        manifests,
        versionName,
      });
    }
  }

  await writeChildManifests(manifests);

  const generatedAt = new Date().toISOString();
  await writeFile(
    path.join(outputDir, 'manifest.json'),
    `${JSON.stringify(createManifestIndex({generatedAt, lastVersionName, manifests}), null, 2)}\n`,
    'utf8',
  );
}

async function exportTree({
  locale,
  lastVersionName,
  sourceDir,
  outputSubdir,
  manifests,
  versionName,
}) {
  const files = await collectMarkdownFiles(sourceDir);
  const docs = [];

  for (const file of files) {
    const relativePath = path.relative(sourceDir, file);
    const outputPath = path.join(outputDir, outputSubdir, relativePath);
    await mkdir(path.dirname(outputPath), {recursive: true});
    await cp(file, outputPath);

    docs.push(
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

  docs.sort((a, b) => a.permalink.localeCompare(b.permalink));
  manifests.set(getManifestKey(locale, versionName), docs);
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
  const docPath = normalizeDocPath(relativePath);
  const content = await readFile(file, 'utf8');
  const parsedDoc = parseDocContent(content, file);

  const entry = {
    title: parsedDoc.title,
    description: parsedDoc.description,
    rawUrl,
    permalink: toPermalink({locale, docPath, lastVersionName, versionName}),
  };

  if (parsedDoc.apis.length > 0) {
    entry.apis = parsedDoc.apis;
  }

  if (parsedDoc.userdatas.length > 0) {
    entry.userdatas = parsedDoc.userdatas;
  }

  return entry;
}

async function writeChildManifests(manifests) {
  for (const [key, docs] of manifests.entries()) {
    const {locale, version} = parseManifestKey(key);
    const manifestPath = path.join(outputDir, manifestOutputSubdir, locale, `${version}.json`);

    await mkdir(path.dirname(manifestPath), {recursive: true});
    await writeFile(
      manifestPath,
      `${JSON.stringify({locale, version, docs}, null, 2)}\n`,
      'utf8',
    );
  }
}

function createManifestIndex({generatedAt, lastVersionName, manifests}) {
  const localeIndex = {};

  for (const key of Array.from(manifests.keys()).sort()) {
    const {locale, version} = parseManifestKey(key);
    const versionIndex = (localeIndex[locale] ??= {});
    versionIndex[version] = `/${toPosixPath(
      path.join('raw', manifestOutputSubdir, locale, `${version}.json`),
    )}`;
  }

  return {
    generatedAt,
    defaultLocale: 'en',
    currentVersion: currentVersionName,
    currentVersionPath,
    latestStableVersion: lastVersionName,
    manifests: localeIndex,
  };
}

function getManifestKey(locale, version) {
  return `${locale}::${version}`;
}

function parseManifestKey(key) {
  const [locale, version] = key.split('::');
  return {locale, version};
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
  return extractTitleFromContent(content, file);
}

function extractTitleFromContent(content, file = 'document') {
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

function parseDocContent(content, file) {
  const title = extractTitleFromContent(content, file);
  const markdown = stripFrontmatter(content);
  const tree = parseMarkdownSections(markdown);
  const docSection = tree.children.find((child) => child.level === 1) ?? tree;
  const description =
    extractSummaryFromLines(docSection.contentLines, {maxParagraphs: 2}) ?? title;
  const apis = extractApis(docSection);
  const userdatas = extractUserdatas(docSection);

  return {title, description, apis, userdatas};
}

function stripFrontmatter(content) {
  return content.replace(/^---\n[\s\S]*?\n---\n*/u, '');
}

function parseMarkdownSections(content) {
  const root = createSectionNode({level: 0, title: ''});
  const stack = [root];
  const lines = content.split(/\r?\n/u);
  let inCodeBlock = false;
  let codeFenceMarker = null;

  for (const line of lines) {
    const fenceMatch = line.match(/^(```+|~~~+)/u);
    if (fenceMatch) {
      const marker = fenceMatch[1][0];
      if (!inCodeBlock) {
        inCodeBlock = true;
        codeFenceMarker = marker;
      } else if (codeFenceMarker === marker) {
        inCodeBlock = false;
        codeFenceMarker = null;
      }
      stack.at(-1).contentLines.push(line);
      continue;
    }

    if (!inCodeBlock) {
      const headingMatch = line.match(/^(#{1,6})\s+(.+)$/u);
      if (headingMatch) {
        const level = headingMatch[1].length;
        const title = headingMatch[2].trim();
        while (stack.at(-1).level >= level) {
          stack.pop();
        }

        const node = createSectionNode({level, title});
        stack.at(-1).children.push(node);
        stack.push(node);
        continue;
      }
    }

    stack.at(-1).contentLines.push(line);
  }

  return root;
}

function createSectionNode({level, title}) {
  return {
    level,
    title,
    contentLines: [],
    children: [],
  };
}

function extractApis(docSection) {
  const apis = [];

  for (const section of getChildSections(docSection, 2)) {
    if (isModuleApiHeading(section.title)) {
      apis.push({
        fullName: section.title.trim(),
        description:
          extractSummaryFromLines(section.contentLines, {maxParagraphs: 2}) ??
          section.title.trim(),
      });
    }

    apis.push(...extractApiLinksFromLines(section.contentLines));
  }

  return dedupeByFullName(apis);
}

function extractUserdatas(docSection) {
  const userdatas = [];

  for (const section of getChildSections(docSection, 2)) {
    if (isModuleApiHeading(section.title)) {
      continue;
    }

    const methods = extractMethods(section);
    if (methods.length === 0 && !containsUserdataSignal(section.contentLines)) {
      continue;
    }

    const fullName = deriveUserdataFullName(section, methods);
    if (!fullName) {
      continue;
    }

    userdatas.push({
      fullName,
      description:
        extractSummaryFromLines(section.contentLines, {
          maxParagraphs: 2,
          skipPatterns: USERDATA_BOILERPLATE_PATTERNS,
        }) ?? section.title.trim(),
      methods,
    });
  }

  return dedupeByFullName(userdatas);
}

function getChildSections(section, level) {
  return section.children.filter((child) => child.level === level);
}

function isModuleApiHeading(title) {
  return /^ptool\./u.test(title.trim());
}

function extractApiLinksFromLines(lines) {
  const apis = [];

  for (const item of collectListItems(lines)) {
    const match = item.match(/^\[([^\]]+)\]\([^)]+\)[:：]\s*(.+)$/u);
    if (!match) {
      continue;
    }

    const fullName = normalizeInlineMarkdown(match[1]);
    if (!/(?:^| )(?:Lua )?API$/u.test(fullName)) {
      continue;
    }

    apis.push({
      fullName,
      description: normalizeInlineMarkdown(match[2]),
    });
  }

  return apis;
}

function extractMethods(section) {
  return getChildSections(section, section.level + 1)
    .map((methodSection) => {
      const fullName = extractCanonicalApiName(methodSection.contentLines);
      if (!fullName) {
        return null;
      }

      return {
        fullName,
        description:
          extractSummaryFromLines(methodSection.contentLines, {maxParagraphs: 2}) ??
          methodSection.title.trim(),
      };
    })
    .filter(Boolean);
}

function extractCanonicalApiName(lines) {
  for (const line of lines) {
    if (!/API/u.test(line)) {
      continue;
    }

    const match = line.match(/`([^`]+:[^`]+)`/u);
    if (match) {
      return match[1].trim();
    }
  }

  return null;
}

function deriveUserdataFullName(section, methods) {
  const firstMethodName = methods[0]?.fullName;
  if (firstMethodName?.includes(':')) {
    return firstMethodName.split(':')[0];
  }

  return section.title.trim() || null;
}

function containsUserdataSignal(lines) {
  return lines.some((line) =>
    /userdata|用户数据|方法：|Methods:/iu.test(line),
  );
}

function extractSummaryFromLines(lines, {maxParagraphs = 1, skipPatterns = []} = {}) {
  const paragraphs = collectParagraphs(lines);
  const accepted = [];

  for (const paragraph of paragraphs) {
    if (shouldSkipParagraph(paragraph, skipPatterns)) {
      continue;
    }

    accepted.push(paragraph);
    if (accepted.length >= maxParagraphs) {
      break;
    }
  }

  if (accepted.length > 0) {
    return accepted.join(' ');
  }

  const fallback = paragraphs.find((paragraph) => paragraph.length > 0);
  return fallback ?? null;
}

function collectParagraphs(lines) {
  const paragraphs = [];
  let block = [];
  let inCodeBlock = false;
  let codeFenceMarker = null;

  const flush = () => {
    if (block.length === 0) {
      return;
    }

    const paragraph = normalizeInlineMarkdown(block.join(' ').trim());
    if (paragraph) {
      paragraphs.push(paragraph);
    }
    block = [];
  };

  for (const rawLine of lines) {
    const line = rawLine.trim();
    const fenceMatch = rawLine.match(/^(```+|~~~+)/u);
    if (fenceMatch) {
      const marker = fenceMatch[1][0];
      if (!inCodeBlock) {
        flush();
        inCodeBlock = true;
        codeFenceMarker = marker;
      } else if (codeFenceMarker === marker) {
        inCodeBlock = false;
        codeFenceMarker = null;
      }
      continue;
    }

    if (inCodeBlock) {
      continue;
    }

    if (!line) {
      flush();
      continue;
    }

    block.push(line);
  }

  flush();
  return paragraphs;
}

function shouldSkipParagraph(paragraph, skipPatterns) {
  if (!paragraph) {
    return true;
  }

  if (
    /^(?:[-*]|\d+\.)\s/u.test(paragraph) ||
    paragraph.startsWith('>') ||
    paragraph.startsWith('|') ||
    paragraph.endsWith(':') ||
    paragraph.endsWith('：')
  ) {
    return true;
  }

  return [...STRUCTURAL_LABEL_PATTERNS, ...skipPatterns].some((pattern) =>
    pattern.test(paragraph),
  );
}

function normalizeInlineMarkdown(value) {
  return value
    .replace(/\[([^\]]+)\]\([^)]+\)/gu, '$1')
    .replace(/`([^`]+)`/gu, '$1')
    .replace(/\*\*([^*]+)\*\*/gu, '$1')
    .replace(/\*([^*]+)\*/gu, '$1')
    .replace(/\s+/gu, ' ')
    .trim();
}

function collectListItems(lines) {
  const items = [];
  let current = null;
  let inCodeBlock = false;
  let codeFenceMarker = null;

  const flush = () => {
    if (!current) {
      return;
    }
    items.push(current.trim());
    current = null;
  };

  for (const rawLine of lines) {
    const fenceMatch = rawLine.match(/^(```+|~~~+)/u);
    if (fenceMatch) {
      const marker = fenceMatch[1][0];
      if (!inCodeBlock) {
        flush();
        inCodeBlock = true;
        codeFenceMarker = marker;
      } else if (codeFenceMarker === marker) {
        inCodeBlock = false;
        codeFenceMarker = null;
      }
      continue;
    }

    if (inCodeBlock) {
      continue;
    }

    const trimmed = rawLine.trim();
    if (!trimmed) {
      flush();
      continue;
    }

    const bulletMatch = rawLine.match(/^\s*-\s+(.*)$/u);
    if (bulletMatch) {
      flush();
      current = bulletMatch[1];
      continue;
    }

    if (current && /^\s{2,}\S/u.test(rawLine)) {
      current = `${current} ${trimmed}`;
      continue;
    }

    flush();
  }

  flush();
  return items;
}

function dedupeByFullName(items) {
  const seen = new Set();
  const deduped = [];

  for (const item of items) {
    if (!item?.fullName || seen.has(item.fullName)) {
      continue;
    }
    seen.add(item.fullName);
    deduped.push(item);
  }

  return deduped;
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

const USERDATA_BOILERPLATE_PATTERNS = [
  /^It is implemented as a Lua userdata\.$/u,
  /^It is implemented as a Lua userdata\.$/iu,
  /^它被实现为一个 Lua userdata。$/u,
  /^它实现为 Lua userdata。$/u,
  /^它实现为 Lua 用户数据（userdata）。$/u,
];

const STRUCTURAL_LABEL_PATTERNS = [
  /^Canonical API name:/iu,
  /^规范 API 名称：/u,
  /^(?:Arguments|Notes|Example|Examples|Behavior|Methods|Fields|Command argument rules|Reply conversion rules):$/iu,
  /^(?:参数|说明|示例|行为说明|方法|字段|支持的模式|返回值规则|通用行为|类型映射|阶段规则|支持的数据库|字段和方法|元方法)：$/u,
];

await main();
