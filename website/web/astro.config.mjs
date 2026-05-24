import { defineConfig } from 'astro/config';
import mdx from '@astrojs/mdx';
import sitemap from '@astrojs/sitemap';
import mermaid from 'astro-mermaid';

// Static output. Cloudflare Pages serves dist/ directly; no adapter needed
// for fully static builds. Switch to @astrojs/cloudflare if/when SSR is added.
export default defineConfig({
  site: 'https://www.deepcausality.com',
  output: 'static',
  integrations: [
    mermaid({
      theme: 'dark',
      autoTheme: true,
    }),
    mdx(),
    sitemap({
      changefreq: 'weekly',
      serialize(item) {
        const url = new URL(item.url);
        const path = url.pathname;
        if (path === '/' || path === '') {
          item.priority = 1.0;
          item.changefreq = 'weekly';
        } else if (path.startsWith('/blog/') && path !== '/blog/') {
          item.priority = 0.8;
          item.changefreq = 'monthly';
        } else if (path.startsWith('/docs/')) {
          item.priority = 0.7;
          item.changefreq = 'monthly';
        } else if (path.startsWith('/examples/')) {
          item.priority = 0.6;
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
    routing: {
      prefixDefaultLocale: false,
    },
  },
  markdown: {
    shikiConfig: {
      // Dual themes; the CSS in global.css toggles between them based on
      // `[data-theme]` on <html>. `defaultColor: false` makes Shiki emit
      // `--shiki-light` / `--shiki-dark` custom properties instead of
      // picking one theme as a hard default.
      themes: {
        light: 'github-light',
        dark: 'github-dark',
      },
      defaultColor: 'dark',
      wrap: true,
    },
  },
});
