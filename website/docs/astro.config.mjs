import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import sitemap from '@astrojs/sitemap';
// Graph view (task 2.2): use starlight-site-graph directly rather than via
// starlight-theme-obsidian@0.4.1, whose `removeDefault().default({})` pattern
// breaks site-graph's nested `z.map()` defaults ("expected map, received
// object"). Used directly, site-graph applies its own valid defaults.
import starlightSiteGraph from 'starlight-site-graph';

// Standalone Starlight documentation site for DeepCausality.
// Served at https://docs.deepcausality.com by its own Cloudflare Worker,
// independent of the marketing site in website/web. Fully static output;
// Cloudflare serves dist/ directly, so no adapter is needed.
export default defineConfig({
  site: 'https://docs.deepcausality.com',
  output: 'static',
  integrations: [
    starlight({
      title: 'DeepCausality',
      description:
        'Documentation for DeepCausality, a Rust framework for computational causality, dynamic causal reasoning, and the Effect Propagation Process.',
      logo: {
        src: './src/assets/logo-color.png',
        alt: 'DeepCausality',
        replacesTitle: false,
      },
      favicon: '/favicon.ico',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/deepcausality-rs/deep_causality' },
      ],
      // A link back to the marketing site (SEO cross-origin link, task 6.4).
      // Starlight renders editLink/social; the explicit www link lives in the
      // sidebar config below and in the index page.
      customCss: [
        './src/styles/fonts.css',
        // site-graph base styles, then our identity tokens last so they win.
        'starlight-site-graph/styles/common.css',
        'starlight-site-graph/styles/starlight.css',
        './src/styles/theme.css',
      ],
      // Code highlighting (task 2.1): dual light/dark, matching the marketing
      // site's Shiki themes in website/web/astro.config.mjs.
      expressiveCode: {
        themes: ['github-dark', 'github-light'],
        styleOverrides: { borderRadius: '0.25rem' },
      },
      // Graph view (task 2.2): backlinks graph in the page sidebar.
      plugins: [starlightSiteGraph()],
      sidebar: [
        {
          label: 'Overview',
          items: [{ autogenerate: { directory: 'overview' } }],
        },
        {
          label: 'Getting Started',
          items: [{ autogenerate: { directory: 'getting-started' } }],
        },
        {
          label: 'Concepts',
          items: [{ autogenerate: { directory: 'concepts' } }],
        },
        {
          label: 'deepcausality.com',
          link: 'https://www.deepcausality.com',
          attrs: { target: '_self' },
        },
      ],
    }),
    sitemap(),
  ],
});
