import { defineConfig } from 'astro/config';
import mdx from '@astrojs/mdx';
import sitemap from '@astrojs/sitemap';

// Static output. Cloudflare Workers serves dist/ directly; no adapter needed.
//
// Deliberately leaner than the marketing site (website/web): no mermaid and no
// pagefind. Diagrams here are hand-drawn SVG in the site's own instrument
// vocabulary (DESIGN.md §12), and search is not wired anywhere in the project
// yet (DESIGN.md §8.9) — shipping an unread index was not worth repeating.
export default defineConfig({
  site: 'https://cfd.deepcausality.com',
  output: 'static',
  integrations: [
    mdx(),
    sitemap({
      changefreq: 'weekly',
      serialize(item) {
        const path = new URL(item.url).pathname;
        if (path === '/' || path === '') {
          item.priority = 1.0;
        } else if (path.startsWith('/validation/')) {
          // The validation-status page is the adoption document; rank it high.
          item.priority = 0.9;
          item.changefreq = 'monthly';
        } else if (path.startsWith('/cookbook/')) {
          item.priority = 0.8;
          item.changefreq = 'monthly';
        } else if (path.startsWith('/examples/')) {
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
      themes: { light: 'github-light', dark: 'github-dark' },
      defaultColor: 'dark',
      wrap: true,
    },
  },
});
