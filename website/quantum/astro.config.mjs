import { defineConfig } from 'astro/config';
import mdx from '@astrojs/mdx';
import sitemap from '@astrojs/sitemap';
import { rustDark, rustLight } from './shiki-rust-themes.mjs';

// Static output. Cloudflare Workers serves dist/ directly; no adapter needed.
//
// Same lean configuration as website/cfd: no mermaid and no pagefind. Figures
// here are HTML and CSS with small inline SVG glyphs in the site's own
// instrument vocabulary (DESIGN.md §12), and search is not wired anywhere in
// the project yet (DESIGN.md §8.9).
export default defineConfig({
  site: 'https://quantum.deepcausality.com',
  output: 'static',

  // Off on purpose. The default compressor strips the line break between a line
  // of text and a following inline element or {expression}, so "rerun,\n{n} steps"
  // renders "rerun,12 steps". Prose on this site is written with ordinary line
  // breaks, so the compressor cannot stay on.
  compressHTML: false,

  // Astro 7.2. Static output with no adapter, so the session runtime is already
  // tree-shaken; declaring it keeps `Astro.session` undefined by contract rather
  // than by inference.
  session: false,

  // The layer-by-layer pages became the pages on the right. A redirect keeps a
  // link that predates the rewrite pointing at the nearest content. `/gates/`
  // has no page: the kernels are documented on docs.rs.
  redirects: {
    '/qcm/': '/how-it-works/',
    '/operators/': '/checks/#channel',
    '/verdicts/': '/checks/#verdicts',
    '/gates/': 'https://docs.rs/deep_causality_quantum',
    '/modalities/': '/boundaries/#modalities',
    '/formalization/': '/proof/',
    '/papers/': '/proof/#papers',
    '/errors/': '/how-it-works/#failure',
  },

  // Astro 7.2 experimental. Skips re-rendering static pages whose module graph
  // and `cacheKey` are unchanged since the last build. The cache lives in
  // `cacheDir` (node_modules/.astro), so it only pays off where that directory
  // survives between builds.
  experimental: {
    incrementalBuild: true,
  },
  integrations: [
    mdx(),
    sitemap({
      changefreq: 'weekly',
      serialize(item) {
        const path = new URL(item.url).pathname;
        if (path === '/' || path === '') {
          item.priority = 1.0;
        } else if (path.startsWith('/how-it-works/')) {
          // The page that says what the library does and how.
          item.priority = 0.9;
          item.changefreq = 'monthly';
        } else if (path.startsWith('/examples/') || path.startsWith('/checks/')) {
          item.priority = 0.85;
          item.changefreq = 'monthly';
        } else if (path.startsWith('/proof/')) {
          // The evidence document.
          item.priority = 0.8;
          item.changefreq = 'monthly';
        } else if (path.startsWith('/boundaries/')) {
          item.priority = 0.75;
          item.changefreq = 'monthly';
        } else if (path.startsWith('/start/')) {
          item.priority = 0.7;
          item.changefreq = 'monthly';
        } else {
          item.priority = 0.4;
          item.changefreq = 'monthly';
        }
        return item;
      },
    }),
  ],
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
    routing: { prefixDefaultLocale: false },
  },
  markdown: {
    shikiConfig: {
      // Dual themes; global.css toggles between them on [data-theme].
      // Ayu-derived, contrast-corrected for --bg-2. See shiki-rust-themes.mjs.
      themes: { light: rustLight, dark: rustDark },
      defaultColor: 'dark',
      wrap: false,
    },
  },
  vite: {
    build: {
      rolldownOptions: {
        // Astro 7.3.5 opens every MDX `?astroPropagatedAssets` module with the directive
        // "use astro:head-inject". Rolldown drops directives it does not know and warns once per
        // MDX file. Nothing reads that directive: Astro finds those modules by the query flag
        // (core/head-propagation/boundary.js), so dropping it changes nothing. Only that one
        // warning is silenced; every other log passes through.
        onLog(level, log, defaultHandler) {
          if (log.code === 'MODULE_LEVEL_DIRECTIVE' && log.message.includes('astro:head-inject')) {
            return;
          }
          defaultHandler(level, log);
        },
      },
    },
  },
});
