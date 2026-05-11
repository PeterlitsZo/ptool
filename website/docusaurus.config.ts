import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import type {PluginOptions as LocalSearchOptions} from '@easyops-cn/docusaurus-search-local';

const localSearchOptions = {
  hashed: true,
  indexBlog: false,
  indexPages: true,
  language: ['en', 'zh', 'es', 'pt', 'ja'],
  searchBarPosition: 'right',
  highlightSearchTermsOnTargetPage: true,
  explicitSearchResultPath: true,
} satisfies LocalSearchOptions;

const localSearchTheme: NonNullable<Config['themes']>[number] = [
  '@easyops-cn/docusaurus-search-local',
  localSearchOptions,
];

const config: Config = {
  title: 'ptool',
  tagline: 'Lua-powered scripting for shell and automation workflows.',
  favicon: 'img/ptool-avatar.svg',

  future: {
    v4: true,
  },

  url: 'https://ptool.peterlits.net',
  baseUrl: '/',
  organizationName: 'PeterlitsZo',
  projectName: 'ptool',
  onBrokenLinks: 'throw',

  i18n: {
    defaultLocale: 'en',
    locales: ['en', 'zh-Hans', 'es', 'pt-BR', 'ja'],
    localeConfigs: {
      en: {
        htmlLang: 'en',
        label: 'English',
      },
      'zh-Hans': {
        htmlLang: 'zh-CN',
        label: '简体中文',
      },
      es: {
        htmlLang: 'es',
        label: 'Español',
      },
      'pt-BR': {
        htmlLang: 'pt-BR',
        label: 'Português (Brasil)',
      },
      ja: {
        htmlLang: 'ja-JP',
        label: '日本語',
      },
    },
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/PeterlitsZo/ptool/tree/main/website/',
          lastVersion: '0.5.0',
          versions: {
            current: {
              label: 'Unreleased',
              path: 'unreleased',
            },
            '0.5.0': {
              label: 'v0.5.0',
            },
          },
        },
        blog: false,
      } satisfies Preset.Options,
    ],
  ],

  themes: [localSearchTheme],

  themeConfig: {
    image: 'img/ptool-avatar.svg',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'ptool',
      logo: {
        alt: 'ptool avatar',
        src: 'img/ptool-avatar.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Docs',
        },
        {
          type: 'localeDropdown',
          position: 'left',
        },
        {
          type: 'docsVersionDropdown',
          position: 'left',
        },
        {
          href: 'https://github.com/PeterlitsZo/ptool/blob/main/CHANGELOG.md',
          label: 'Changelog',
          position: 'right',
        },
        {
          href: 'https://github.com/PeterlitsZo/ptool',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Getting Started',
              to: '/docs/intro',
            },
          ],
        },
        {
          title: 'Reference',
          items: [
            {
              label: 'Lua API Overview',
              to: '/docs/lua-api',
            },
            {
              label: 'Changelog',
              href: 'https://github.com/PeterlitsZo/ptool/blob/main/CHANGELOG.md',
            },
          ],
        },
        {
          title: 'Project',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/PeterlitsZo/ptool',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} ptool.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
