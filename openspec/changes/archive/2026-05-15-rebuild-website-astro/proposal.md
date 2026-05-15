## Why

The current deepcausality.com site is built with Hugo and has become unmaintainable: its theming, content pipeline, and shortcode system resist routine updates, and most of its documentation is outdated relative to the present state of the 20-crate monorepo. At the same time, the project's framing as "hypergeometric computational causality" has consistently failed to land with engineers — landing-page analytics-by-observation suggest visitors skip the philosophical preamble. The site needs both a technical and editorial reset before the next round of outreach.

## What Changes

- **BREAKING**: Stand up a new Astro site at `website/web/` to replace the existing Hugo-based deepcausality.com, which lives in a separate repository. `ctx/` in this monorepo is a one-time snapshot of Markdown content and images copied from that external repo — it is a migration source only, not a live site.
- **BREAKING**: Rebrand the framework's public framing from "hypergeometric computational causality" to "dynamic causality" across all site copy, taglines, meta tags, and navigation.
- Redesign the landing page to lead with **six clickable code examples** drawn from distinct engineering domains, each linking to a dedicated elaboration page. Philosophical/conceptual material moves below the fold and into the docs section.
- Replace the bulk of existing documentation with newly authored docs sourced from the monograph in `papers/` and from the current state of the crates. Only the blog and a curated subset of pages migrate from Hugo content.
- Adopt Astro's Markdown-driven i18n. Scaffold the locale structure (English only at launch) so additional locales can be added without restructuring.
- Host on Cloudflare (Pages, with the option of Workers for any future dynamic endpoints).
- Move the front-page hero art (`ctx/static/img/frontpage-art.webp`) and other static assets into the Astro `public/` tree under conventional paths.

## Capabilities

### New Capabilities
- `website-platform`: The Astro project itself — framework setup, build/deploy pipeline to Cloudflare, i18n scaffolding, base layouts, and shared components.
- `landing-page`: Code-example-first landing page with six domain examples, each linking to a detail page; rebranded hero and above-the-fold copy.
- `documentation-site`: Newly authored documentation section sourced from the monograph and the crates, replacing the legacy Hugo docs.
- `content-migration`: One-time migration of the blog and selected static pages/assets from `ctx/` into the Astro content collections, plus retirement plan for `ctx/`.
- `brand-identity`: Site-wide rebrand from "hypergeometric computational causality" to "dynamic causality" — taglines, meta tags, navigation labels, and a glossary defining the new framing.

### Modified Capabilities
<!-- No existing OpenSpec specs to modify; this is the first website-related change. -->

## Impact

- **New top-level directory**: `website/web/` (Astro project, Node/pnpm toolchain).
- **Migration source**: `ctx/` — read-only snapshot of Markdown/images from the external Hugo repo. Once migration is complete it can be removed (requires user approval per repo Golden Rules); the upstream Hugo repo is unaffected by this change.
- **Hosting/DNS**: deepcausality.com DNS and Cloudflare Pages project must be reconfigured to point at the new Astro build. The existing Hugo deployment (in its separate repo) remains the live site until cutover.
- **CI**: No new workflow added. Cloudflare Pages already auto-deploys this repo (fork branches → beta domain, `main` → production). Cutover is an operator-side update of the Pages project's source repo / root directory to `website/web/`.
- **Authoring**: All future docs and blog posts authored as Markdown/MDX under `website/web/src/content/`.
- **No impact** on the Rust crates themselves — this change is purely web-facing.
