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
      theme: 'github-dark',
      wrap: true,
    },
  },
});
