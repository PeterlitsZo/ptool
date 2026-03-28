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
      items: ['lua-api/index', 'lua-api/reference'],
    },
  ],
};

export default sidebars;
