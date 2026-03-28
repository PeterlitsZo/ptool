import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';

import styles from './index.module.css';

const features = [
  {
    title: 'Script-first',
    body:
      'Write plain Lua files and run them with ptool. The runtime exposes utilities under both ptool and p.',
  },
  {
    title: 'Batteries included',
    body:
      'Use helpers for semver, files, HTTP, SSH, templates, databases, and text processing without assembling separate tools.',
  },
  {
    title: 'CLI-friendly',
    body:
      'Mix shell commands, argument parsing, shebang scripts, and structured APIs in one place for practical automation.',
  },
];

const script = `ptool.use("v0.1.0")

local who = ptool.ask("Deploy target?", {
  default = "staging",
})

ptool.run("echo", {"deploying", who})`;

function HomepageHeader(): ReactNode {
  const {siteConfig} = useDocusaurusContext();

  return (
    <header className={styles.heroBanner}>
      <div className={clsx('container', styles.heroShell)}>
        <div className={styles.heroCopy}>
          <div className={styles.eyebrow}>Docs</div>
          <Heading as="h1" className={styles.heroTitle}>
            {siteConfig.title}
          </Heading>
          <p className={styles.heroSubtitle}>{siteConfig.tagline}</p>
          <p className={styles.heroBody}>
            Build small, sharp automation scripts with Lua, then ship them like
            command-line tools.
          </p>
          <div className={styles.buttons}>
            <Link className="button button--primary button--lg" to="/docs/intro">
              Get Started
            </Link>
            <Link
              className={clsx(
                'button button--lg button--outline button--secondary',
                styles.secondaryButton,
              )}
              to="/docs/lua-api/reference">
              Lua API
            </Link>
          </div>
        </div>
        <div className={styles.heroPanel}>
          <div className={styles.panelLabel}>example.lua</div>
          <pre className={styles.codeBlock}>
            <code>{script}</code>
          </pre>
          <p className={styles.panelNote}>
            Start with the guide, then use the full API reference as your
            scripting manual.
          </p>
        </div>
      </div>
    </header>
  );
}

function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.section}>
      <div className="container">
        <div className={styles.sectionHeading}>
          <p className={styles.sectionKicker}>Why ptool</p>
          <Heading as="h2">A practical runtime for automation scripts</Heading>
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

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();

  return (
    <Layout
      title={siteConfig.title}
      description="Documentation for ptool, a Lua-powered scripting tool for shell and automation workflows.">
      <HomepageHeader />
      <main>
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
