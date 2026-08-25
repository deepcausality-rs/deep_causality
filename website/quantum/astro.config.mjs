import { defineConfig } from 'astro/config';
import mdx from '@astrojs/mdx';
import sitemap from '@astrojs/sitemap';
import { rustDark, rustLight } from './shiki-rust-themes.mjs';

// Static output. Cloudflare Workers serves dist/ directly; no adapter needed.
//
// Same lean configuration as website/cfd: no mermaid and no pagefind. Diagrams
// here are hand-drawn SVG in the site's own instrument vocabulary
// (DESIGN.md §12), and search is not wired anywhere in the project yet
// (DESIGN.md §8.9).
export default defineConfig({
  site: 'https://quantum.deepcausality.com',
  output: 'static',

  // Astro 7.2. Static output with no adapter, so the session runtime is already
  // tree-shaken; declaring it keeps `Astro.session` undefined by contract rather
  // than by inference.
  session: false,

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
        } else if (path.startsWith('/qcm/')) {
          // The quantum causal model is what the crate is for; everything else
          // is a layer under it.
          item.priority = 0.9;
          item.changefreq = 'monthly';
        } else if (path.startsWith('/formalization/')) {
          // The Lean status page is the evidence document.
          item.priority = 0.9;
          item.changefreq = 'monthly';
        } else if (path.startsWith('/operators/')) {
          item.priority = 0.85;
          item.changefreq = 'monthly';
        } else if (
          path.startsWith('/gates/') ||
          path.startsWith('/verdicts/') ||
          path.startsWith('/modalities/')
        ) {
          item.priority = 0.8;
          item.changefreq = 'monthly';
        } else if (path.startsWith('/examples/')) {
          item.priority = 0.75;
          item.changefreq = 'monthly';
        } else if (path.startsWith('/papers/') || path.startsWith('/start/')) {
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
      wrap: true,
    },
  },
});
