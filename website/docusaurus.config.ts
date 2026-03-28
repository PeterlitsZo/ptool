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

  url: 'https://peterlitszo.github.io',
  baseUrl: '/ptool/',
  organizationName: 'PeterlitsZo',
  projectName: 'ptool',
  onBrokenLinks: 'throw',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
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
              label: 'Introduction',
              to: '/docs/intro',
            },
          ],
        },
        {
          title: 'Reference',
          items: [
            {
              label: 'Lua API',
              to: '/docs/lua-api/reference',
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
