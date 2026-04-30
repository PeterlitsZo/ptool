import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import Translate, {translate} from '@docusaurus/Translate';
import useBaseUrl from '@docusaurus/useBaseUrl';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';

import styles from './index.module.css';

const script = `ptool.use("v0.1.0")

local who = ptool.ask("Deploy target?", {
  default = "staging",
})

ptool.run("echo", {"deploying", who})`;

function getFeatures() {
  return [
    {
      title: translate({
        id: 'homepage.features.scriptFirst.title',
        message: 'Script-first',
      }),
      body: translate({
        id: 'homepage.features.scriptFirst.body',
        message:
          'Write plain Lua files and run them with ptool. The runtime exposes utilities under both ptool and p.',
      }),
    },
    {
      title: translate({
        id: 'homepage.features.batteriesIncluded.title',
        message: 'Batteries included',
      }),
      body: translate({
        id: 'homepage.features.batteriesIncluded.body',
        message:
          'Use helpers for semver, files, HTTP, SSH, templates, databases, and text processing without assembling separate tools.',
      }),
    },
    {
      title: translate({
        id: 'homepage.features.cliFriendly.title',
        message: 'CLI-friendly',
      }),
      body: translate({
        id: 'homepage.features.cliFriendly.body',
        message:
          'Mix shell commands, argument parsing, shebang scripts, and structured APIs in one place for practical automation.',
      }),
    },
  ];
}

function HomepageHeader(): ReactNode {
  const {siteConfig} = useDocusaurusContext();

  return (
    <header className={styles.heroBanner}>
      <div className={clsx('container', styles.heroShell)}>
        <div className={styles.heroCopy}>
          <div className={styles.eyebrow}>
            <Translate id="homepage.eyebrow">Docs</Translate>
          </div>
          <Heading as="h1" className={styles.heroTitle}>
            {siteConfig.title}
          </Heading>
          <p className={styles.heroSubtitle}>
            <Translate id="homepage.tagline">
              Lua-powered scripting for shell and automation workflows.
            </Translate>
          </p>
          <p className={styles.heroBody}>
            <Translate id="homepage.heroBody">
              Build small, sharp automation scripts with Lua, then ship them
              like command-line tools.
            </Translate>
          </p>
          <div className={styles.buttons}>
            <Link className="button button--primary button--lg" to="/docs/intro">
              <Translate id="homepage.cta.getStarted">Get Started</Translate>
            </Link>
            <Link
              className={clsx(
                'button button--lg button--outline button--secondary',
                styles.secondaryButton,
              )}
              to="/docs/lua-api">
              <Translate id="homepage.cta.luaApi">Lua API</Translate>
            </Link>
          </div>
        </div>
        <div className={styles.heroPanel}>
          <div className={styles.panelLabel}>example.lua</div>
          <pre className={styles.codeBlock}>
            <code>{script}</code>
          </pre>
          <p className={styles.panelNote}>
            <Translate id="homepage.panelNote">
              Start with the guide, then jump into the module-based Lua API
              docs as your scripting manual.
            </Translate>
          </p>
        </div>
      </div>
    </header>
  );
}

function HomepageFeatures(): ReactNode {
  const features = getFeatures();

  return (
    <section className={styles.section}>
      <div className="container">
        <div className={styles.sectionHeading}>
          <p className={styles.sectionKicker}>
            <Translate id="homepage.sectionKicker">Why ptool</Translate>
          </p>
          <Heading as="h2">
            <Translate id="homepage.sectionTitle">
              A practical runtime for automation scripts
            </Translate>
          </Heading>
        </div>
        <div className={styles.featureGrid}>
          {features.map(feature => (
            <article key={feature.title} className={styles.featureCard}>
              <Heading as="h3">{feature.title}</Heading>
              <p>{feature.body}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}

function HomepageAiAccess(): ReactNode {
  const manifestHref = useBaseUrl('/raw/manifest.json');
  const introRawHref = useBaseUrl('/raw/docs/intro.md');
  const zhIntroRawHref = useBaseUrl('/raw/i18n/zh-Hans/docs/intro.md');

  return (
    <section className={clsx(styles.section, styles.aiSection)}>
      <div className="container">
        <div className={styles.sectionHeading}>
          <p className={styles.sectionKicker}>
            <Translate id="homepage.ai.kicker">For AI</Translate>
          </p>
          <Heading as="h2">
            <Translate id="homepage.ai.title">
              Give assistants raw Markdown instead of rendered HTML
            </Translate>
          </Heading>
          <p className={styles.aiIntro}>
            <Translate id="homepage.ai.intro">
              ptool publishes the source docs as static files so agents can read
              the original Markdown directly.
            </Translate>
          </p>
        </div>
        <div className={styles.aiGrid}>
          <article className={styles.aiCard}>
            <Heading as="h3">
              <Translate id="homepage.ai.manifest.title">1. Start here</Translate>
            </Heading>
            <p>
              <Translate id="homepage.ai.manifest.body">
                Tell your assistant to fetch the manifest first. It lists every
                document title, locale, permalink, and raw Markdown URL.
              </Translate>
            </p>
            <a className={styles.inlineLink} href={manifestHref}>
              /raw/manifest.json
            </a>
          </article>
          <article className={styles.aiCard}>
            <Heading as="h3">
              <Translate id="homepage.ai.raw.title">2. Open one page</Translate>
            </Heading>
            <p>
              <Translate id="homepage.ai.raw.body">
                Each document also has a stable raw Markdown URL. Use the
                manifest, or point to a page directly.
              </Translate>
            </p>
            <div className={styles.aiExamples}>
              <a className={styles.inlineLink} href={introRawHref}>
                /raw/docs/intro.md
              </a>
              <a className={styles.inlineLink} href={zhIntroRawHref}>
                /raw/i18n/zh-Hans/docs/intro.md
              </a>
            </div>
          </article>
          <article className={styles.aiCard}>
            <Heading as="h3">
              <Translate id="homepage.ai.prompt.title">
                3. Tell the model what to do
              </Translate>
            </Heading>
            <p>
              <Translate id="homepage.ai.prompt.body">
                Example: read the manifest, select the matching locale and
                page, then load the raw Markdown instead of scraping the
                rendered site.
              </Translate>
            </p>
            <pre className={styles.aiPrompt}>
              <code>
                {`Fetch /raw/manifest.json, find the page for "/docs/intro", then read its rawUrl.`}
              </code>
            </pre>
          </article>
        </div>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();

  return (
    <Layout
      title={siteConfig.title}
      description={translate({
        id: 'homepage.description',
        message:
          'Documentation for ptool, a Lua-powered scripting tool for shell and automation workflows.',
      })}>
      <HomepageHeader />
      <main>
        <HomepageFeatures />
        <HomepageAiAccess />
      </main>
    </Layout>
  );
}
