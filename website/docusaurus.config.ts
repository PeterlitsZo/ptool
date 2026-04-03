import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'ptool',
  tagline: 'Lua-powered scripting for shell and automation workflows.',
  favicon: 'img/favicon.ico',

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
    locales: ['en', 'zh-Hans'],
    localeConfigs: {
      en: {
        htmlLang: 'en',
      },
      'zh-Hans': {
        htmlLang: 'zh-CN',
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
        },
        blog: false,
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/favicon.ico',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'ptool',
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
