import {execFile as execFileCallback} from 'node:child_process';
import {access, mkdir, readdir, readFile, rm, rmdir, writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {promisify} from 'node:util';
import {fileURLToPath} from 'node:url';

const execFile = promisify(execFileCallback);

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const websiteDir = path.resolve(__dirname, '..');
const docsDir = path.join(websiteDir, 'docs');
const versionedDocsDir = path.join(websiteDir, 'versioned_docs');
const i18nDir = path.join(websiteDir, 'i18n');
const poDocsDir = path.join(websiteDir, 'po', 'docs');
const poTemplatesDir = path.join(poDocsDir, 'templates');
const docsI18nLockPath = path.join(poDocsDir, 'docs-i18n.lock.json');
const docsPluginDirName = 'docusaurus-plugin-content-docs';
const sourceDocFilePattern = /\.(md|markdown|txt|text)$/;
const poCatalogFilePattern = /\.(po|pot)$/;
const command = process.argv[2] ?? 'help';

await main();

async function main() {
  switch (command) {
    case 'sync':
      await ensureToolkitDependencies();
      await syncCatalogs();
      break;
    case 'compile':
      await ensureToolkitDependencies();
      await compileCatalogs();
      break;
    case 'refresh':
      await ensureToolkitDependencies();
      await syncCatalogs();
      await compileCatalogs();
      break;
    case 'help':
    default:
      printHelp();
      if (command !== 'help') {
        process.exitCode = 1;
      }
      break;
  }
}

function printHelp() {
  console.log(`docs-i18n commands:
  sync     Extract source docs into POT/PO catalogs.
  compile  Render locale Markdown from PO catalogs.
  refresh  Run sync and compile in sequence.`);
}

async function ensureToolkitDependencies() {
  try {
    await execFile(
      'python3',
      [
        '-c',
        'import mistletoe.token; import translate.convert.md2po; import translate.convert.po2md; import translate.convert.pot2po',
      ],
      {cwd: websiteDir},
    );
  } catch (error) {
    console.error(
      'Translate Toolkit dependencies are missing. Install them with:\n' +
        '  python3 -m pip install --user -r website/requirements-docs-i18n.txt',
    );
    throw error;
  }
}

async function syncCatalogs() {
  const locales = await readLocales();
  const docSets = await readDocSets();
  const previousLock = await readDocsI18nLock();
  const nextLock = {
    version: 1,
    templates: {},
    locales: {},
  };
  const docSetNames = new Set(docSets.map((docSet) => docSet.name));

  for (const docSet of docSets) {
    nextLock.templates[docSet.name] = await extractTemplateDocSet({
      docSet,
      previousHashes: previousLock.templates?.[docSet.name] ?? {},
    });
  }

  for (const locale of locales) {
    nextLock.locales[locale] = {};
    for (const docSet of docSets) {
      nextLock.locales[locale][docSet.name] = await syncLocaleDocSet({
        locale,
        docSet,
        previousHashes: previousLock.locales?.[locale]?.[docSet.name] ?? {},
      });
    }
  }

  await removeStaleDocSetDirs(poTemplatesDir, docSetNames);
  await removeStaleLocaleDirs(poDocsDir, new Set(locales));
  for (const locale of locales) {
    await removeStaleDocSetDirs(path.join(poDocsDir, locale), docSetNames);
  }

  await writeDocsI18nLock(nextLock);
}

async function compileCatalogs() {
  const locales = await readLocales();
  const docSets = await readDocSets();
  const preparedTemplateDocSets = new Set();

  for (const locale of locales) {
    for (const docSet of docSets) {
      await ensureLocaleDocSetForCompile({
        locale,
        docSet,
        preparedTemplateDocSets,
      });
      await compileLocaleDocSet({locale, docSet});
    }
  }
}

async function ensureLocaleDocSetForCompile({
  locale,
  docSet,
  preparedTemplateDocSets,
}) {
  const localePoDir = path.join(poDocsDir, locale, docSet.name);
  if (await pathExists(localePoDir)) {
    return;
  }

  const templateDir = path.join(poTemplatesDir, docSet.name);
  if (!preparedTemplateDocSets.has(docSet.name) && !(await pathExists(templateDir))) {
    await extractTemplateDocSet({docSet, previousHashes: {}});
  }

  preparedTemplateDocSets.add(docSet.name);
  await syncLocaleDocSet({locale, docSet, previousHashes: {}});
}

async function extractTemplateDocSet({docSet, previousHashes}) {
  const templateDir = path.join(poTemplatesDir, docSet.name);
  await mkdir(templateDir, {recursive: true});
  const sourceFiles = await collectFiles(docSet.sourceDir, sourceDocFilePattern);
  const nextHashes = {};
  const expectedOutputFiles = new Set();

  for (const relativePath of sourceFiles) {
    const sourcePath = path.join(docSet.sourceDir, relativePath);
    const outputRelativePath = replaceExtension(relativePath, '.pot');
    const outputPath = path.join(templateDir, outputRelativePath);
    const hash = await hashFile(sourcePath);

    nextHashes[relativePath] = hash;
    expectedOutputFiles.add(outputRelativePath);

    if (previousHashes[relativePath] === hash && (await pathExists(outputPath))) {
      continue;
    }

    await mkdir(path.dirname(outputPath), {recursive: true});
    await runPythonModule('translate.convert.md2po', [
      '--progress=none',
      '--duplicates=msgctxt',
      '-P',
      '-i',
      sourcePath,
      '-o',
      outputPath,
    ]);
  }

  await removeUnexpectedFiles(templateDir, expectedOutputFiles, poCatalogFilePattern);

  console.log(`extracted ${path.relative(websiteDir, templateDir)}`);
  return nextHashes;
}

async function syncLocaleDocSet({locale, docSet, previousHashes}) {
  const templateDir = path.join(poTemplatesDir, docSet.name);
  const localePoDir = path.join(poDocsDir, locale, docSet.name);
  await mkdir(localePoDir, {recursive: true});
  const templateFiles = await collectFiles(templateDir, /\.pot$/);
  const nextHashes = {};
  const expectedOutputFiles = new Set();

  for (const relativePath of templateFiles) {
    const templatePath = path.join(templateDir, relativePath);
    const outputRelativePath = replaceExtension(relativePath, '.po');
    const outputPath = path.join(localePoDir, outputRelativePath);
    const hash = await hashFile(templatePath);

    nextHashes[relativePath] = hash;
    expectedOutputFiles.add(outputRelativePath);

    if (previousHashes[relativePath] === hash && (await pathExists(outputPath))) {
      continue;
    }

    const previousContent = (await pathExists(outputPath)) ? await readFile(outputPath, 'utf8') : null;

    await mkdir(path.dirname(outputPath), {recursive: true});
    const args = [
      '--progress=none',
      '-i',
      templatePath,
      '-o',
      outputPath,
    ];
    if (await pathExists(outputPath)) {
      args.push('-t', outputPath);
    }
    await runPythonModule('translate.convert.pot2po', args);
    await restoreMovedTranslations(outputPath, previousContent);
  }

  await removeUnexpectedFiles(localePoDir, expectedOutputFiles, /\.po$/);

  console.log(`synced ${path.relative(websiteDir, localePoDir)}`);
  return nextHashes;
}

async function compileLocaleDocSet({locale, docSet}) {
  const localePoDir = path.join(poDocsDir, locale, docSet.name);
  const targetDir = path.join(i18nDir, locale, docsPluginDirName, docSet.name);

  await rm(targetDir, {recursive: true, force: true});
  await mkdir(path.dirname(targetDir), {recursive: true});

  await runPythonModule('translate.convert.po2md', [
    '--progress=none',
    '--maxlinelength=0',
    '-i',
    localePoDir,
    '-t',
    docSet.sourceDir,
    '-o',
    targetDir,
  ]);

  console.log(`compiled ${locale}/${docSet.name}`);
}

async function restoreMovedTranslations(poPath, previousContent) {
  const currentContent = await readFile(poPath, 'utf8');
  const currentDoc = parsePoDocument(currentContent);
  const previousDoc = previousContent ? parsePoDocument(previousContent) : {entries: []};
  const translationLookup = buildTranslationLookup([
    ...previousDoc.entries,
    ...currentDoc.entries.filter((entry) => entry.obsolete),
  ]);

  let changed = false;

  for (const entry of currentDoc.entries) {
    if (entry.obsolete || entry.msgid === null) {
      continue;
    }

    const previousEntry = translationLookup.get(entry.msgid);
    if (!previousEntry || !previousEntry.msgstr) {
      continue;
    }

    if (!entry.flags.has('fuzzy') && entry.msgstr) {
      continue;
    }

    if (entry.msgstr === previousEntry.msgstr && !entry.flags.has('fuzzy')) {
      continue;
    }

    entry.msgstr = previousEntry.msgstr;
    entry.flags.delete('fuzzy');
    changed = true;
  }

  if (changed) {
    await writeFile(poPath, serializePoDocument(currentDoc));
  }
}

function buildTranslationLookup(entries) {
  const lookup = new Map();

  for (const entry of entries) {
    if (entry.msgid === null || !entry.msgstr) {
      continue;
    }
    if (!lookup.has(entry.msgid)) {
      lookup.set(entry.msgid, entry);
    }
  }

  return lookup;
}

function parsePoDocument(content) {
  const normalized = content.replace(/\r\n/g, '\n');
  const blocks = normalized.split(/\n{2,}/);
  const entries = [];

  for (const block of blocks) {
    if (!block.trim()) {
      continue;
    }
    entries.push(parsePoEntry(block));
  }

  return {entries};
}

function parsePoEntry(block) {
  const lines = block.split('\n');
  const comments = [];
  const flags = new Set();
  let obsolete = false;
  let currentField = null;
  let msgctxt = null;
  let msgid = null;
  let msgstr = null;

  for (const rawLine of lines) {
    if (!rawLine) {
      continue;
    }

    let line = rawLine;
    let lineObsolete = false;
    if (line.startsWith('#~ ')) {
      line = line.slice(3);
      lineObsolete = true;
      obsolete = true;
    }

    if (line.startsWith('#,')) {
      for (const flag of line.slice(2).split(',')) {
        const normalizedFlag = flag.trim();
        if (normalizedFlag) {
          flags.add(normalizedFlag);
        }
      }
      continue;
    }

    if (line.startsWith('#')) {
      comments.push(rawLine);
      continue;
    }

    if (line.startsWith('msgctxt ')) {
      msgctxt = parsePoStringLiteral(line.slice(8));
      currentField = 'msgctxt';
      continue;
    }

    if (line.startsWith('msgid ')) {
      msgid = parsePoStringLiteral(line.slice(6));
      currentField = 'msgid';
      continue;
    }

    if (line.startsWith('msgstr ')) {
      msgstr = parsePoStringLiteral(line.slice(7));
      currentField = 'msgstr';
      continue;
    }

    if (line.startsWith('"')) {
      const value = parsePoStringLiteral(line);
      if (currentField === 'msgctxt') {
        msgctxt = (msgctxt ?? '') + value;
      } else if (currentField === 'msgid') {
        msgid = (msgid ?? '') + value;
      } else if (currentField === 'msgstr') {
        msgstr = (msgstr ?? '') + value;
      }
      continue;
    }

    if (lineObsolete) {
      comments.push(rawLine);
      continue;
    }

    comments.push(rawLine);
  }

  return {
    comments,
    flags,
    obsolete,
    msgctxt,
    msgid,
    msgstr: msgstr ?? '',
  };
}

function parsePoStringLiteral(source) {
  return JSON.parse(source);
}

function serializePoDocument(document) {
  return `${document.entries.map(serializePoEntry).join('\n\n')}\n`;
}

function serializePoEntry(entry) {
  const lines = [...entry.comments];

  if (entry.flags.size > 0) {
    lines.push(`#, ${[...entry.flags].join(', ')}`);
  }

  const prefix = entry.obsolete ? '#~ ' : '';

  if (entry.msgctxt !== null) {
    lines.push(...formatPoField('msgctxt', entry.msgctxt, prefix));
  }
  if (entry.msgid !== null) {
    lines.push(...formatPoField('msgid', entry.msgid, prefix));
  }
  lines.push(...formatPoField('msgstr', entry.msgstr, prefix));

  return lines.join('\n');
}

function formatPoField(name, value, prefix = '') {
  if (!value.includes('\n')) {
    return [`${prefix}${name} ${JSON.stringify(value)}`];
  }

  return [
    `${prefix}${name} ""`,
    ...splitPoMultilineValue(value).map((part) => `${prefix}${JSON.stringify(part)}`),
  ];
}

function splitPoMultilineValue(value) {
  const parts = [];
  let start = 0;

  while (start < value.length) {
    const nextNewline = value.indexOf('\n', start);
    if (nextNewline === -1) {
      parts.push(value.slice(start));
      break;
    }

    parts.push(value.slice(start, nextNewline + 1));
    start = nextNewline + 1;
  }

  if (parts.length === 0) {
    parts.push('');
  }

  return parts;
}

async function runPythonModule(moduleName, args) {
  await execFile('python3', ['-m', moduleName, ...args], {
    cwd: websiteDir,
    maxBuffer: 32 * 1024 * 1024,
  });
}

async function readDocsI18nLock() {
  try {
    const content = await readFile(docsI18nLockPath, 'utf8');
    const parsed = JSON.parse(content);
    return {
      version: parsed?.version ?? 1,
      templates: parsed?.templates ?? {},
      locales: parsed?.locales ?? {},
    };
  } catch (error) {
    if (error && typeof error === 'object' && 'code' in error && error.code === 'ENOENT') {
      return {
        version: 1,
        templates: {},
        locales: {},
      };
    }
    throw error;
  }
}

async function writeDocsI18nLock(lock) {
  await mkdir(path.dirname(docsI18nLockPath), {recursive: true});
  await writeFile(docsI18nLockPath, `${JSON.stringify(lock, null, 2)}\n`);
}

async function collectFiles(rootDir, pattern) {
  if (!(await pathExists(rootDir))) {
    return [];
  }

  const files = [];
  await collectFilesInto(rootDir, rootDir, pattern, files);
  files.sort((a, b) => a.localeCompare(b));
  return files;
}

async function collectFilesInto(rootDir, currentDir, pattern, files) {
  const entries = await readdir(currentDir, {withFileTypes: true});

  for (const entry of entries) {
    const entryPath = path.join(currentDir, entry.name);

    if (entry.isDirectory()) {
      await collectFilesInto(rootDir, entryPath, pattern, files);
      continue;
    }

    if (!entry.isFile() || !pattern.test(entry.name)) {
      continue;
    }

    files.push(path.relative(rootDir, entryPath));
  }
}

async function hashFile(filePath) {
  const content = await readFile(filePath);
  return createHash('sha256').update(content).digest('hex');
}

function replaceExtension(relativePath, nextExtension) {
  const extension = path.extname(relativePath);
  return `${relativePath.slice(0, -extension.length)}${nextExtension}`;
}

async function removeUnexpectedFiles(rootDir, expectedFiles, pattern) {
  if (!(await pathExists(rootDir))) {
    return;
  }

  const existingFiles = await collectFiles(rootDir, pattern);

  for (const relativePath of existingFiles) {
    if (expectedFiles.has(relativePath)) {
      continue;
    }

    await rm(path.join(rootDir, relativePath), {force: true});
  }

  await removeEmptyDirectories(rootDir);
}

async function removeStaleDocSetDirs(rootDir, expectedDirNames) {
  if (!(await pathExists(rootDir))) {
    return;
  }

  const entries = await readdir(rootDir, {withFileTypes: true});

  for (const entry of entries) {
    const entryPath = path.join(rootDir, entry.name);

    if (!entry.isDirectory() || expectedDirNames.has(entry.name)) {
      continue;
    }

    await rm(entryPath, {recursive: true, force: true});
  }
}

async function removeStaleLocaleDirs(rootDir, expectedLocaleNames) {
  if (!(await pathExists(rootDir))) {
    return;
  }

  const entries = await readdir(rootDir, {withFileTypes: true});

  for (const entry of entries) {
    const entryPath = path.join(rootDir, entry.name);

    if (!entry.isDirectory()) {
      continue;
    }

    if (entry.name === 'templates' || expectedLocaleNames.has(entry.name)) {
      continue;
    }

    await rm(entryPath, {recursive: true, force: true});
  }
}

async function removeEmptyDirectories(rootDir) {
  const entries = await readdir(rootDir, {withFileTypes: true});

  for (const entry of entries) {
    if (!entry.isDirectory()) {
      continue;
    }

    const entryPath = path.join(rootDir, entry.name);
    await removeEmptyDirectories(entryPath);
    const childEntries = await readdir(entryPath);
    if (childEntries.length === 0) {
      await rmdir(entryPath);
    }
  }
}

async function readLocales() {
  const entries = await readdir(i18nDir, {withFileTypes: true});
  return entries
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort((a, b) => a.localeCompare(b));
}

async function readDocSets() {
  const versions = await readVersionNames();

  return [
    {name: 'current', sourceDir: docsDir},
    ...versions.map((version) => ({
      name: `version-${version}`,
      sourceDir: path.join(versionedDocsDir, `version-${version}`),
    })),
  ];
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

async function pathExists(targetPath) {
  try {
    await access(targetPath);
    return true;
  } catch (error) {
    if (error && typeof error === 'object' && 'code' in error && error.code === 'ENOENT') {
      return false;
    }
    throw error;
  }
}
