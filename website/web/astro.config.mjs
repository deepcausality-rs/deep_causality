import { defineConfig } from 'astro/config';
import mdx from '@astrojs/mdx';
import sitemap from '@astrojs/sitemap';

// Static output. Cloudflare Pages serves dist/ directly; no adapter needed
// for fully static builds. Switch to @astrojs/cloudflare if/when SSR is added.
export default defineConfig({
  site: 'https://deepcausality.com',
  output: 'static',
  integrations: [mdx(), sitemap()],
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
