import {execFile as execFileCallback} from 'node:child_process';
import {
  access,
  mkdtemp,
  mkdir,
  readdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import os from 'node:os';
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
const docsPluginDirName = 'docusaurus-plugin-content-docs';
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
  sync     Extract source docs into POT/PO catalogs and import current translations.
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

  await rm(poTemplatesDir, {recursive: true, force: true});

  for (const docSet of docSets) {
    await extractTemplateDocSet(docSet);
  }

  for (const locale of locales) {
    for (const docSet of docSets) {
      await syncLocaleDocSet({locale, docSet});
    }
  }
}

async function compileCatalogs() {
  const locales = await readLocales();
  const docSets = await readDocSets();

  for (const locale of locales) {
    for (const docSet of docSets) {
      await compileLocaleDocSet({locale, docSet});
    }
  }
}

async function extractTemplateDocSet(docSet) {
  const templateDir = path.join(poTemplatesDir, docSet.name);
  await rm(templateDir, {recursive: true, force: true});
  await mkdir(templateDir, {recursive: true});

  await runPythonModule('translate.convert.md2po', [
    '--progress=none',
    '--duplicates=msgctxt',
    '--multifile=toplevel',
    '-P',
    '-i',
    docSet.sourceDir,
    '-o',
    templateDir,
  ]);

  console.log(`extracted ${path.relative(websiteDir, templateDir)}`);
}

async function syncLocaleDocSet({locale, docSet}) {
  const templateDir = path.join(poTemplatesDir, docSet.name);
  const localePoDir = path.join(poDocsDir, locale, docSet.name);
  const localeDocsDir = path.join(i18nDir, locale, docsPluginDirName, docSet.name);

  await mkdir(localePoDir, {recursive: true});

  await runPythonModule('translate.convert.pot2po', [
    '--progress=none',
    '-i',
    templateDir,
    '-o',
    localePoDir,
    '-t',
    localePoDir,
  ]);

  await importExistingTranslations({
    locale,
    localeDocsDir,
    localePoDir,
    templateDir,
  });

  console.log(`synced ${path.relative(websiteDir, localePoDir)}`);
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

async function importExistingTranslations({
  locale,
  localeDocsDir,
  localePoDir,
  templateDir,
}) {
  if (!(await pathExists(localeDocsDir))) {
    return;
  }

  const tempDir = await mkdtemp(path.join(os.tmpdir(), `ptool-i18n-${locale}-`));

  try {
    await runPythonModule('translate.convert.md2po', [
      '--progress=none',
      '--duplicates=msgctxt',
      '--multifile=toplevel',
      '-i',
      localeDocsDir,
      '-o',
      tempDir,
    ]);

    const poFiles = await collectPoFiles(localePoDir);
    for (const poFile of poFiles) {
      const relativePath = path.relative(localePoDir, poFile);
      const templatePath = path.join(templateDir, relativePath.replace(/\.po$/u, '.pot'));
      const existingPath = path.join(tempDir, relativePath);

      if (!(await pathExists(templatePath)) || !(await pathExists(existingPath))) {
        continue;
      }

      await importPoFile({
        locale,
        poPath: poFile,
        templatePath,
        existingPath,
      });
    }
  } finally {
    await rm(tempDir, {recursive: true, force: true});
  }
}

async function importPoFile({locale, poPath, templatePath, existingPath}) {
  const poCatalog = parsePo(await readFile(poPath, 'utf8'));
  const templateCatalog = parsePo(await readFile(templatePath, 'utf8'));
  const existingCatalog = parsePo(await readFile(existingPath, 'utf8'));

  const poEntries = poCatalog.entries.filter(isTranslatableEntry);
  const templateEntries = templateCatalog.entries.filter(isTranslatableEntry);
  const existingEntries = existingCatalog.entries.filter(isTranslatableEntry);

  if (poEntries.length === 0 || templateEntries.length === 0 || existingEntries.length === 0) {
    return;
  }

  const sourceRecords = await buildPoRecords(templateEntries);
  const targetRecords = await buildPoRecords(existingEntries);
  const alignedPairs =
    sourceRecords.length === targetRecords.length
      ? sourceRecords.map((_, index) => [index, index])
      : alignPoRecords(sourceRecords, targetRecords);

  let changed = false;

  for (const [sourceIndex, targetIndex] of alignedPairs) {
    const poEntry = poEntries[sourceIndex];
    const translatedEntry = existingEntries[targetIndex];
    const sourceRecord = sourceRecords[sourceIndex];
    const targetRecord = targetRecords[targetIndex];

    if (!poEntry || !translatedEntry) {
      continue;
    }
    if (poEntry.msgstr.trim()) {
      continue;
    }
    if (scoreRecordMatch(sourceRecord, targetRecord) < 4) {
      continue;
    }

    poEntry.msgstr = translatedEntry.msgid;
    changed = true;
  }

  if (!changed) {
    return;
  }

  poCatalog.header.msgstr = ensureLanguageHeader(poCatalog.header.msgstr, locale);
  await writeFile(poPath, serializePo(poCatalog), 'utf8');
}

async function buildPoRecords(entries) {
  return Promise.all(
    entries.map(async (entry) => {
      const reference = entry.references[0] ?? '';
      const parsedReference = parseReference(reference);
      const lineText = parsedReference ? await readReferenceLine(parsedReference) : '';

      return {
        blockType: classifyReferenceLine(lineText),
        codeSpanCount: countMatches(entry.msgid, /`[^`]+`/gu),
        lineText,
        lineTextLength: lineText.trim().length,
        numberTokenCount: countMatches(entry.msgid, /\b\d+(?:\.\d+)*\b/gu),
        placeholderCount: countMatches(entry.msgid, /\{\d+\}/gu),
        prefix: classifyPrefix(lineText),
      };
    }),
  );
}

function alignPoRecords(sourceRecords, targetRecords) {
  const rows = sourceRecords.length + 1;
  const cols = targetRecords.length + 1;
  const scores = Array.from({length: rows}, () => Array(cols).fill(0));
  const steps = Array.from({length: rows}, () => Array(cols).fill(''));

  for (let i = 1; i < rows; i += 1) {
    scores[i][0] = scores[i - 1][0] - 3;
    steps[i][0] = 'up';
  }
  for (let j = 1; j < cols; j += 1) {
    scores[0][j] = scores[0][j - 1] - 3;
    steps[0][j] = 'left';
  }

  for (let i = 1; i < rows; i += 1) {
    for (let j = 1; j < cols; j += 1) {
      const diagonal =
        scores[i - 1][j - 1] + scoreRecordMatch(sourceRecords[i - 1], targetRecords[j - 1]);
      const up = scores[i - 1][j] - 3;
      const left = scores[i][j - 1] - 3;

      if (diagonal >= up && diagonal >= left) {
        scores[i][j] = diagonal;
        steps[i][j] = 'diag';
      } else if (up >= left) {
        scores[i][j] = up;
        steps[i][j] = 'up';
      } else {
        scores[i][j] = left;
        steps[i][j] = 'left';
      }
    }
  }

  const aligned = [];
  let i = sourceRecords.length;
  let j = targetRecords.length;

  while (i > 0 && j > 0) {
    const step = steps[i][j];
    if (step === 'diag') {
      aligned.push([i - 1, j - 1]);
      i -= 1;
      j -= 1;
    } else if (step === 'up') {
      i -= 1;
    } else {
      j -= 1;
    }
  }

  return aligned.reverse();
}

function scoreRecordMatch(sourceRecord, targetRecord) {
  let score = 0;

  if (sourceRecord.blockType === targetRecord.blockType) {
    score += 6;
  } else {
    score -= 6;
  }

  if (sourceRecord.prefix === targetRecord.prefix) {
    score += 2;
  }

  if (sourceRecord.placeholderCount === targetRecord.placeholderCount) {
    score += 3;
  }

  if (sourceRecord.codeSpanCount === targetRecord.codeSpanCount) {
    score += 2;
  }

  if (sourceRecord.numberTokenCount === targetRecord.numberTokenCount) {
    score += 2;
  }

  if (
    sourceRecord.lineTextLength > 0 &&
    targetRecord.lineTextLength > 0 &&
    Math.abs(sourceRecord.lineTextLength - targetRecord.lineTextLength) <= 24
  ) {
    score += 1;
  }

  return score;
}

function classifyReferenceLine(lineText) {
  const trimmed = lineText.trimStart();

  if (/^#{1,6}\s+/u.test(trimmed)) {
    return 'heading';
  }
  if (/^>\s?/u.test(trimmed)) {
    return 'blockquote';
  }
  if (/^([-*+]|\d+\.)\s+/u.test(trimmed)) {
    return 'list';
  }
  return 'paragraph';
}

function classifyPrefix(lineText) {
  const trimmed = lineText.trimStart();
  const line = trimmed.split('\n', 1)[0] ?? '';

  if (/^#{1,6}\s+/u.test(line)) {
    return line.match(/^#{1,6}/u)?.[0] ?? '#';
  }
  if (/^\d+\.\s+/u.test(line)) {
    return 'ordered-list';
  }
  if (/^[-*+]\s+/u.test(line)) {
    return 'unordered-list';
  }
  if (/^>\s?/u.test(line)) {
    return 'blockquote';
  }
  return 'paragraph';
}

function countMatches(text, pattern) {
  return text.match(pattern)?.length ?? 0;
}

function parseReference(reference) {
  const separatorIndex = reference.lastIndexOf(':');
  if (separatorIndex === -1) {
    return null;
  }

  const filePath = reference.slice(0, separatorIndex);
  const lineNumber = Number.parseInt(reference.slice(separatorIndex + 1), 10);

  if (!filePath || !Number.isInteger(lineNumber) || lineNumber < 1) {
    return null;
  }

  return {filePath, lineNumber};
}

async function readReferenceLine({filePath, lineNumber}) {
  try {
    const content = await readFile(path.resolve(websiteDir, '..', filePath), 'utf8');
    const lines = content.replace(/\r\n/g, '\n').split('\n');
    return lines[lineNumber - 1] ?? '';
  } catch {
    return '';
  }
}

async function runPythonModule(moduleName, args) {
  await execFile('python3', ['-m', moduleName, ...args], {
    cwd: websiteDir,
    maxBuffer: 32 * 1024 * 1024,
  });
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

async function collectPoFiles(dir) {
  if (!(await pathExists(dir))) {
    return [];
  }

  const entries = await readdir(dir, {withFileTypes: true});
  const files = await Promise.all(
    entries.map(async (entry) => {
      const entryPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        return collectPoFiles(entryPath);
      }
      if (entry.isFile() && entry.name.endsWith('.po')) {
        return [entryPath];
      }
      return [];
    }),
  );

  return files.flat().sort((a, b) => a.localeCompare(b));
}

function parsePo(content) {
  const lines = content.replace(/\r\n/g, '\n').split('\n');
  const entries = [];
  let entry = createEmptyPoEntry();
  let field = null;

  for (const line of lines) {
    if (!line.trim()) {
      if (hasPoContent(entry)) {
        entries.push(entry);
        entry = createEmptyPoEntry();
        field = null;
      }
      continue;
    }

    if (line.startsWith('#. ')) {
      entry.comments.push(line.slice(3));
      continue;
    }
    if (line.startsWith('#, ')) {
      entry.flags.push(
        ...line
          .slice(3)
          .split(',')
          .map((value) => value.trim())
          .filter(Boolean),
      );
      continue;
    }
    if (line.startsWith('#: ')) {
      entry.references.push(...line.slice(3).trim().split(/\s+/u).filter(Boolean));
      continue;
    }
    if (line.startsWith('msgctxt ')) {
      field = 'msgctxt';
      entry.msgctxt = parsePoFragment(line.slice('msgctxt '.length));
      continue;
    }
    if (line.startsWith('msgid ')) {
      field = 'msgid';
      entry.msgid = parsePoFragment(line.slice('msgid '.length));
      continue;
    }
    if (line.startsWith('msgstr ')) {
      field = 'msgstr';
      entry.msgstr = parsePoFragment(line.slice('msgstr '.length));
      continue;
    }
    if (line.startsWith('"') && field) {
      entry[field] += parsePoFragment(line);
    }
  }

  if (hasPoContent(entry)) {
    entries.push(entry);
  }

  const [header = createHeaderEntry(), ...restEntries] = entries;
  return {
    entries: restEntries,
    header: header.msgid === '' ? header : createHeaderEntry(),
  };
}

function createHeaderEntry() {
  return {
    comments: [],
    flags: [],
    msgctxt: '',
    msgid: '',
    msgstr:
      'Project-Id-Version: PACKAGE VERSION\n' +
      'MIME-Version: 1.0\n' +
      'Content-Type: text/plain; charset=UTF-8\n' +
      'Content-Transfer-Encoding: 8bit\n',
    references: [],
  };
}

function createEmptyPoEntry() {
  return {
    comments: [],
    flags: [],
    msgctxt: '',
    msgid: '',
    msgstr: '',
    references: [],
  };
}

function hasPoContent(entry) {
  return (
    entry.comments.length > 0 ||
    entry.flags.length > 0 ||
    entry.references.length > 0 ||
    entry.msgctxt ||
    entry.msgid ||
    entry.msgstr
  );
}

function isTranslatableEntry(entry) {
  return entry.msgid !== '';
}

function parsePoFragment(fragment) {
  return JSON.parse(fragment);
}

function serializePo(catalog) {
  const sections = [serializePoEntry(catalog.header)];

  for (const entry of catalog.entries) {
    sections.push(serializePoEntry(entry));
  }

  return `${sections.join('\n\n')}\n`;
}

function serializePoEntry(entry) {
  const lines = [];

  for (const comment of entry.comments) {
    lines.push(`#. ${comment}`);
  }
  if (entry.flags.length > 0) {
    lines.push(`#, ${entry.flags.join(', ')}`);
  }
  if (entry.references.length > 0) {
    lines.push(`#: ${entry.references.join(' ')}`);
  }
  if (entry.msgctxt) {
    lines.push(`msgctxt ${JSON.stringify(entry.msgctxt)}`);
  }
  lines.push(`msgid ${JSON.stringify(entry.msgid)}`);
  lines.push(`msgstr ${JSON.stringify(entry.msgstr)}`);

  return lines.join('\n');
}

function ensureLanguageHeader(headerValue, locale) {
  const lines = headerValue.split('\n').filter((line) => line.length > 0);
  let sawLanguage = false;

  const updatedLines = lines.map((line) => {
    if (line.startsWith('Language:')) {
      sawLanguage = true;
      return `Language: ${locale}`;
    }
    return line;
  });

  if (!sawLanguage) {
    updatedLines.push(`Language: ${locale}`);
  }

  return `${updatedLines.join('\n')}\n`;
}

async function pathExists(filePath) {
  try {
    await access(filePath);
    return true;
  } catch {
    return false;
  }
}
