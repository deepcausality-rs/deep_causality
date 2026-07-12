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
  vite: {
    // starlight-site-graph bundles a glob matcher (picomatch) for its
    // visibilityRules, and picomatch reads process.platform / process.version
    // at module init. Those are undefined in the browser, which threw
    // "process is not defined" and left the graph stuck on its skeleton.
    // Shim the two exact reads for the client bundle (non-Windows, modern
    // version => default glob behavior, lookbehind supported).
    define: {
      'process.platform': JSON.stringify('browser'),
      'process.version': JSON.stringify('v20.0.0'),
    },
  },
  integrations: [
    starlight({
      title: 'DeepCausality',
      description:
        'Documentation for DeepCausality, a Rust framework for computational causality, dynamic causal reasoning, and the Effect Propagation Process.',
      // Full wordmark lockup; white in dark mode, black in light mode (matches
      // the marketing site header). replacesTitle so the title text is not
      // duplicated next to the logo.
      logo: {
        light: './src/assets/logo_black.svg',
        dark: './src/assets/logo_white.svg',
        alt: 'DeepCausality',
        replacesTitle: true,
      },
      favicon: '/favicon.ico',
      // Open the header social links (Discord, GitHub) in a new tab. Starlight
      // renders them in-tab and exposes no per-link attrs, so set it on load.
      head: [
        {
          tag: 'script',
          content:
            "addEventListener('DOMContentLoaded',function(){document.querySelectorAll('.social-icons a').forEach(function(a){a.target='_blank';a.rel='me noopener noreferrer';});});",
        },
        // Default social-share (Open Graph + Twitter) image for every docs page.
        { tag: 'meta', attrs: { property: 'og:image', content: 'https://docs.deepcausality.com/img/social-share.jpg' } },
        { tag: 'meta', attrs: { property: 'og:image:width', content: '1200' } },
        { tag: 'meta', attrs: { property: 'og:image:height', content: '630' } },
        { tag: 'meta', attrs: { property: 'og:image:alt', content: 'DeepCausality' } },
        { tag: 'meta', attrs: { name: 'twitter:card', content: 'summary_large_image' } },
        { tag: 'meta', attrs: { name: 'twitter:image', content: 'https://docs.deepcausality.com/img/social-share.jpg' } },
      ],
      social: [
        { icon: 'discord', label: 'Discord', href: 'https://discord.gg/Bxj9P7JXSj' },
        { icon: 'github', label: 'GitHub', href: 'https://github.com/deepcausality-rs/deep_causality' },
      ],
      // A link back to the marketing site (SEO cross-origin link, task 6.4).
      // Starlight renders editLink/social; the explicit www link lives in the
      // sidebar config below and in the index page.
      // The starlight-site-graph plugin auto-injects its own stylesheets
      // (layers/common/starlight.css), so we only add our identity + splash layers.
      customCss: ['./src/styles/fonts.css', './src/styles/theme.css', './src/styles/splash.css'],
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
          label: 'Formalization',
          items: [{ autogenerate: { directory: 'formalization' } }],
        },
        {
          label: 'Download PDF',
          link: '/deepcausality-docs.pdf',
          attrs: { target: '_blank', rel: 'noopener' },
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
