import {execFile as execFileCallback} from 'node:child_process';
import {mkdir, readdir, readFile, rm} from 'node:fs/promises';
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
