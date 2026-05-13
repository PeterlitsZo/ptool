import {execFile as execFileCallback} from 'node:child_process';
import {readdir, readFile, rm, writeFile} from 'node:fs/promises';
import path from 'node:path';
import {promisify} from 'node:util';
import {fileURLToPath} from 'node:url';

const execFile = promisify(execFileCallback);

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const websiteDir = path.resolve(__dirname, '..');
const versionedDocsDir = path.join(websiteDir, 'versioned_docs');
const versionedSidebarsDir = path.join(websiteDir, 'versioned_sidebars');
const poDocsDir = path.join(websiteDir, 'po', 'docs');
const poTemplatesDir = path.join(poDocsDir, 'templates');
const i18nDir = path.join(websiteDir, 'i18n');

const [command, ...restArgs] = process.argv.slice(2);

await main();

async function main() {
  switch (command) {
    case 'snapshot':
      await runSnapshot(restArgs);
      break;
    case 'prune':
      await runPrune(restArgs);
      break;
    case 'help':
    case undefined:
      printHelp();
      break;
    default:
      console.error(`Unknown command: ${command}`);
      printHelp();
      process.exitCode = 1;
      break;
  }
}

async function runSnapshot(args) {
  const versionName = args[0];
  if (!versionName) {
    throw new Error('snapshot requires a version name.');
  }

  const keep = parseKeepCount(args.slice(1));
  await execFile('npx', ['docusaurus', 'docs:version', versionName], {
    cwd: websiteDir,
    maxBuffer: 32 * 1024 * 1024,
  });
  console.log(`snapshotted docs for ${versionName}`);

  const {keptVersions, prunedVersions} = await pruneVersions({keep});
  printSummary({keptVersions, prunedVersions});
}

async function runPrune(args) {
  const keep = parseKeepCount(args);
  const {keptVersions, prunedVersions} = await pruneVersions({keep});
  printSummary({keptVersions, prunedVersions});
}

function parseKeepCount(args) {
  const keepArgIndex = args.indexOf('--keep');
  if (keepArgIndex === -1) {
    return 3;
  }

  const keepValue = args[keepArgIndex + 1];
  if (!keepValue || !/^\d+$/u.test(keepValue)) {
    throw new Error('--keep expects a non-negative integer.');
  }

  return Number.parseInt(keepValue, 10);
}

async function pruneVersions({keep}) {
  const versionNames = await readVersionNames();
  const sortedVersions = [...versionNames].sort(compareVersionsDesc);
  const keptVersions = sortedVersions.slice(0, keep);
  const prunedVersions = sortedVersions.slice(keep);

  for (const versionName of prunedVersions) {
    await removeVersionArtifacts(versionName);
  }

  await writeVersionNames(keptVersions);
  return {keptVersions, prunedVersions};
}

async function readVersionNames() {
  try {
    const content = await readFile(path.join(websiteDir, 'versions.json'), 'utf8');
    const parsed = JSON.parse(content);
    if (!Array.isArray(parsed)) {
      throw new Error('versions.json must contain an array.');
    }
    return parsed;
  } catch (error) {
    if (error && typeof error === 'object' && 'code' in error && error.code === 'ENOENT') {
      return [];
    }
    throw error;
  }
}

async function writeVersionNames(versionNames) {
  await writeFile(
    path.join(websiteDir, 'versions.json'),
    `${JSON.stringify(versionNames, null, 2)}\n`,
    'utf8',
  );
}

async function removeVersionArtifacts(versionName) {
  const versionDirName = `version-${versionName}`;
  const poLocales = await readLocaleDirs(poDocsDir, ['templates']);
  const i18nLocales = await readLocaleDirs(i18nDir);

  const removalPaths = [
    path.join(versionedDocsDir, versionDirName),
    path.join(versionedSidebarsDir, `${versionDirName}-sidebars.json`),
    path.join(poTemplatesDir, versionDirName),
    ...poLocales.map((locale) => path.join(poDocsDir, locale, versionDirName)),
    ...i18nLocales.map((locale) =>
      path.join(i18nDir, locale, 'docusaurus-plugin-content-docs', versionDirName),
    ),
  ];

  await Promise.all(
    removalPaths.map((targetPath) => rm(targetPath, {recursive: true, force: true})),
  );
}

async function readLocaleDirs(rootDir, excludedNames = []) {
  try {
    const entries = await readdir(rootDir, {withFileTypes: true});
    return entries
      .filter((entry) => entry.isDirectory() && !excludedNames.includes(entry.name))
      .map((entry) => entry.name)
      .sort((a, b) => a.localeCompare(b));
  } catch (error) {
    if (error && typeof error === 'object' && 'code' in error && error.code === 'ENOENT') {
      return [];
    }
    throw error;
  }
}

function compareVersionsDesc(left, right) {
  const leftParts = parseVersion(left);
  const rightParts = parseVersion(right);

  for (let index = 0; index < 3; index += 1) {
    if (leftParts[index] !== rightParts[index]) {
      return rightParts[index] - leftParts[index];
    }
  }

  return 0;
}

function parseVersion(versionName) {
  const match = versionName.match(/^(\d+)\.(\d+)\.(\d+)$/u);
  if (!match) {
    throw new Error(`Unsupported docs version format: ${versionName}`);
  }
  return match.slice(1).map((part) => Number.parseInt(part, 10));
}

function printSummary({keptVersions, prunedVersions}) {
  console.log(`kept versions: ${keptVersions.join(', ') || '(none)'}`);
  console.log(`pruned versions: ${prunedVersions.join(', ') || '(none)'}`);
}

function printHelp() {
  console.log(`manage-doc-versions commands:
  snapshot <version> [--keep <count>]  Snapshot current docs and keep newest versions.
  prune [--keep <count>]               Keep only the newest versions.
  help                                 Show this message.`);
}
