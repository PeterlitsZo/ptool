import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: 'category',
      label: 'Introduction',
      items: ['intro'],
    },
    {
      type: 'category',
      label: 'Guides',
      items: ['getting-started'],
    },
    {
      type: 'category',
      label: 'Lua API',
      items: [
        'lua-api/index',
        'lua-api/core',
        'lua-api/semver',
        'lua-api/hash',
        'lua-api/net',
        'lua-api/platform',
        'lua-api/ansi',
        'lua-api/http',
        'lua-api/db',
        'lua-api/ssh',
        'lua-api/path',
        'lua-api/toml',
        'lua-api/re',
        'lua-api/str',
        'lua-api/fs',
        'lua-api/sh',
        'lua-api/template',
        'lua-api/args',
      ],
    },
  ],
};

export default sidebars;
