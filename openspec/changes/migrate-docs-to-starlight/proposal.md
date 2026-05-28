## Why

The documentation is outgrowing the hand-built Astro marketing site and growing faster than the site around it. Every docs feature (sidebar, search, prev/next, table of contents) is maintained by hand in `website/web`, and a docs edit currently forces a rebuild and redeploy of the entire marketing site. Migrating the long-form documentation to [Starlight](https://starlight.astro.build/) on a dedicated subdomain (`docs.deepcausality.com`) removes that maintenance burden, lets the docs deploy independently of the marketing site, and unlocks documentation features (proper code highlighting, a backlinks graph view, a single build-time PDF) that we would otherwise hand-roll. The design and all six platform decisions are settled in `openspec/notes/Starlight-Docs.md`.

## What Changes

- Add a new, standalone Starlight application in `website/docs` with its own `package.json`, lockfile, Astro version, and `wrangler.toml`, decoupled from `website/web`.
- Migrate the long-form documentation into Starlight: `concepts/*` (13 pages), the deep `getting-started/*` walkthroughs, and the deep `overview/*` pages (including Literature, Innovations, Problem). Map the existing frontmatter (`title`/`description`/`section`/`order`) to Starlight's conventions and rewrite internal links.
- Style the Starlight docs to match the marketing site's visual identity (colors, fonts, logo) so `www` and `docs` read as one product; restyle the Obsidian theme to those tokens.
- Enable documentation features: Expressive Code highlighting (dual light/dark), the `starlight-theme-obsidian` backlinks graph view, and a single-PDF export via a **local** `starlight-to-pdf` script whose committed output is served as a static asset (Cloudflare's build env has no headless browser).
- Deploy the docs from a dedicated Cloudflare Worker (`deepcausality-docs`) bound to `docs.deepcausality.com`, with CI path filters so a change under `website/docs/**` rebuilds only the docs and `website/web/**` rebuilds only the marketing site.
- On the marketing site: keep the short getting-started and short overview, add a new "Documentation" landing page linking to `docs.deepcausality.com` and to the Rust API reference on docs.rs, remove the migrated long-form pages, and drop `/docs/*` from the `www` sitemap.
- Preserve SEO across the split: a Domain property for `deepcausality.com` in Search Console, a per-origin sitemap and `robots.txt` on the docs origin, docs canonicals on the `docs.` origin, two-way cross-linking, and **BREAKING** 301 redirects from `www.deepcausality.com/docs/*` to `https://docs.deepcausality.com/*` (existing `/docs` URLs change origin).
- Examples (`examples/en/*`) stay on `www`; docs↔examples references become cross-origin absolute URLs.

## Capabilities

### New Capabilities
- `documentation-site`: the Starlight documentation application, its migrated content and structure, code highlighting, graph view, and shared visual identity with the marketing site.
- `documentation-deployment`: the standalone `website/docs` build, the dedicated Cloudflare Worker, the `docs.deepcausality.com` origin, and the independent (path-filtered) deploy topology.
- `documentation-seo`: sitemap and `robots.txt` per origin, canonical URLs, the `www/docs/*` → `docs.` 301 redirects, the Search Console Domain property, and cross-origin linking.
- `documentation-pdf`: the local `starlight-to-pdf` build script and the committed single-PDF artifact served statically.
- `marketing-site-docs-handoff`: the `www` retention of short getting-started/overview, the new Documentation landing page linking to the docs subdomain and docs.rs, removal of migrated long-form pages, and `www` sitemap cleanup.

### Modified Capabilities
<!-- None. Existing specs are physics/math/graph capabilities; none govern the website or documentation platform. -->

## Impact

- **New:** `website/docs/` (standalone Starlight app), a `deepcausality-docs` Cloudflare Worker, a `docs.deepcausality.com` DNS record/route, and a Search Console Domain property.
- **Modified:** `website/web` loses the long-form `docs/concepts`, deep `docs/getting-started`, and deep `docs/overview` content and the `src/components/docs` / `src/pages/docs` rendering; gains a Documentation landing page and short summary pages; its sitemap serialization drops `/docs/*`.
- **Dependencies:** new docs app pulls in `@astrojs/starlight`, `starlight-theme-obsidian`, `starlight-to-pdf` (+ its headless-browser tooling, local only). Astro 6 vs Starlight/plugin compatibility must be verified; the docs app can pin its own Astro version independently of `website/web`.
- **SEO / infra:** existing inbound links and indexed `/docs/*` URLs on `www` are redirected (301) to the new origin; two virtual origins served under one Cloudflare zone.
- **CI/CD:** deploy pipeline split by path so the two sites build and ship independently.
