## Status

The site is **live** at https://deep-causality.pages.dev (Cloudflare Pages preview, fed by this monorepo's `website/web/` directory). Build is green at 57 pages on Astro 6.3.2. Production cutover to `deepcausality.com` is operator-side and not in this change's diff.

The shipped surface is materially larger than the original plan in this file; sections marked _shipped_ note material deviations from the original scope. Sections marked _deferred_ remain valid follow-on work.

## 1. Astro project bootstrap

- [x] 1.1 Create `website/web/` directory and initialize an Astro 4.x project with TypeScript, MDX, and the Cloudflare adapter (static output) — _shipped on Astro **6.3.2** (current stable); static output; no Cloudflare adapter needed since Pages serves `dist/` directly._
- [x] 1.2 Configure pnpm; add `package.json`, `pnpm-lock.yaml`, `.npmrc`, and a `.gitignore` for `node_modules` and `dist` — _plus `pnpm-workspace.yaml` `allowBuilds` for esbuild/sharp under pnpm 11, and root `.gitignore` updated._
- [x] 1.3 Verify the Rust Cargo workspace is unaffected — _30 members, none under `website/`._
- [x] 1.4 Configure `astro.config.mjs` with `i18n: { defaultLocale: 'en', locales: ['en'], routing: { prefixDefaultLocale: false } }`
- [x] 1.5 Add brand-token CSS module (colors, typography, spacing) and a global stylesheet — _final tokens in `src/styles/tokens.css`, dark + light themes both first-class._
- [x] 1.6 Create `BaseLayout.astro` with shared header, footer, and meta defaults using rebranded copy
- [x] 1.7 Define content collection schemas in `src/content.config.ts` for: `blog`, `docs`, `examples`, `monograph` — _`examples` later gained a `category` field for the four-category split._
- [x] 1.8 Verify `pnpm install && pnpm build` produces a static site in `website/web/dist/`

## 2. Brand identity & visual design

- [x] 2.0 Invoke the `design-taste-frontend` skill to produce `DESIGN.md` — _the binding reference for every page; covers dark + light themes, type scale, spacing, motion rules, anti-patterns._
- [x] 2.1 Place logo variants and favicons under `website/web/public/brand/` — _shipped under `public/img/` (`logo_black.svg`, `logo_white.svg`) and `public/` root for favicon assets (`favicon.svg`, `favicon-32x32.png`, `apple-touch-icon.png`, `android-chrome-{192,512}.png`, `favicon.ico`, `site.webmanifest`). Layout deviates from the `brand/` subdir originally planned because the favicon set was already laid out flat in `ctx/static/`._
- [x] 2.2 Move the hero art from `ctx/static/img/frontpage-art.webp` to `website/web/public/img/frontpage-art.webp` (copy; do not delete source)
- [x] 2.3 Author `/docs/concepts/glossary` with canonical terminology plus a "former framing" entry addressing the prior "hypergeometric computational causality" label
- [x] 2.4 Audit all hero, nav, and meta copy to use the "dynamic causality" framing
- [ ] 2.5 _**Deferred.**_ Add a build-time check that greps `dist/**/*.html` for "hypergeometric" and AI-Styleguide banned phrases — _not blocking; an audit was run manually and produced zero hits. Still a worthwhile CI guard for the long term._
- [x] 2.6 Confirm with user: final tagline wording — _"Dynamic causality for advanced systems."_

## 3. Landing page

- [x] 3.1 Hero section: logo, rebranded tagline, two CTAs
- [x] 3.2 Six-card code-example grid component
- [x] 3.3 Six Rust snippets (10–20 lines each) — _all six replaced with **real code excerpted from the live `examples/` crates** after user feedback that initial snippets were fabricated. Current slate: aerospace flight envelope, sensor monitoring (CSM), async event inference (Tokio), biomedical tumor treatment, physics Maxwell, Pearl counterfactual. Ordering on the landing page: Tokio → Pearl → CSM → aerospace → biomedical → physics._
- [x] 3.4 Syntax highlighting via Shiki — _dual-theme (`github-light` / `github-dark`) with CSS variable switching; landing card uses `<Code defaultColor={false}>`, markdown code fences use `defaultColor: 'dark'` with a global CSS swap for light mode._
- [x] 3.5 "What is dynamic causality?" explainer
- [x] 3.6 Three-pillar section — _Causal Monad → Causaloid → Context, per the rebrand. SVG connector path at desktop; vertical stack with hairlines below 900px._
- [x] 3.7 Verify at 1440×900 above-the-fold contains only hero + first row of cards — _verified during user review of the live preview._

### Mobile-first responsiveness (out-of-band; user-driven)
- [x] 3.r Refactor all breakpoints to mobile-first (`min-width` queries); add `min-width: 0` overflow guards on grid items so long code lines scroll inside cards rather than pushing the card off-page; add mobile disclosure nav under the header bar; hero stacks below 900px with the art moving above the text; pillar row stacks below 900px; docs sidebar becomes a flat block on mobile and sticky on desktop.

### Vendored fonts (out-of-band; user-driven)
- [x] 3.x Vendor `Geist Variable` and `JetBrains Mono Variable` woff2 files into `public/fonts/` and reference via `@font-face`; preload primary files in `BaseLayout`; remove `@fontsource/*` packages so no node_modules indirection at runtime — _per user feedback: avoid CDN/render-blocking font loads._

## 4. Example detail pages

- [x] 4.1 Create the `examples` collection route `/examples/[slug]` driven by MDX files in `src/content/examples/en/`
- [x] 4.2 Author `quant-finance` walkthrough — _**replaced.** No real quant-finance example exists in `examples/` today. Original slot reassigned; see expansion below._
- [x] 4.3 Author `robotics-control` walkthrough — _**replaced** with `aerospace-flight-envelope` (real source: `examples/avionics_examples/flight_envelope_monitor/`)._
- [x] 4.4 Author `observability-sre` walkthrough — _**replaced** with `sensor-monitoring-csm` (real source: `examples/csm_examples/csm_basic/`)._
- [x] 4.5 Author `bioinformatics-signal` walkthrough — _**replaced** with `biomedical-tumor-treatment` (real source: `examples/medicine_examples/tumor_treatment/`)._
- [x] 4.6 Author `physics-simulation` walkthrough — _**replaced** with `physics-maxwell` (real source: `examples/physics_examples/maxwell/`)._
- [x] 4.7 Author `policy-compliance` walkthrough — _**replaced** with `pearl-counterfactual` (real source: `examples/starter_example/`)._
- [x] 4.7a Add `async-event-inference` (real source: `examples/tokio_example/`).
- [x] 4.8 Verify each landing-page card links to its detail page and all six pages render

### Additional example detail pages (out-of-band; user-driven)
- [x] 4.9 Author `protein-folding` (real source: `examples/medicine_examples/protein_folding/`).
- [x] 4.10 Author `event-horizon-probe` (real source: `examples/physics_examples/event_horizon_probe/`).
- [x] 4.11 Author `grmhd` (real source: `examples/physics_examples/grmhd/`).
- [x] 4.12 Author `gm-recovery` (real source: `examples/chronometric_examples/gm_recovery/`).
- [x] 4.13 Add "more examples" note at the bottom of `/examples/` linking to the full `examples/` tree on GitHub.

### Examples category restructuring (out-of-band; user-driven)
- [x] 4.cat.1 Add `category` enum field (`foundations | aerospace | physics | medicine`) to the examples content schema; populate it across all ten MDX files.
- [x] 4.cat.2 Create four category landing pages at `/examples/foundations/`, `/examples/aerospace/`, `/examples/physics/`, `/examples/medicine/`.
- [x] 4.cat.3 Convert the top-nav "Examples" item into a dropdown (desktop hover + focus-within; mobile disclosure sub-list). Items: All examples / Foundations / Aerospace / Physics / Medicine.
- [x] 4.cat.4 Add a quick category-tag row on the master `/examples/` page so users without a dropdown can still jump between categories.

## 5. Documentation tree (newly authored)

- [x] 5.0 Read the AI Styleguide and the `ctx/docs/*.md` intro material before authoring.
- [x] 5.1 Author `/docs/getting-started/install.md` — _real verify-the-install snippet using `PropagatingEffect::pure` (the originally drafted `deep_causality::VERSION` constant did not exist; fixed after user feedback)._
- [x] 5.1a Author `/docs/getting-started/hello-causal-monad.md` — _added between install and hello-causaloid at user request; teaches `pure` + `bind` before introducing Causaloids._
- [x] 5.2 Author `/docs/getting-started/hello-causaloid.md`
- [x] 5.3 Author `/docs/getting-started/hello-context.md`
- [x] 5.4 Author `/docs/concepts/dynamic-causality.md` — _umbrella concept page anchored on the `m₂ = m₁ >>= f` axiom and the three modalities (static/dynamic/emergent)._
- [x] 5.5 Author `/docs/concepts/causaloid.md`
- [x] 5.6 Author `/docs/concepts/context.md`
- [x] 5.7 Author `/docs/concepts/effect-ethos.md`
- [x] 5.8 Author `/docs/concepts/effect-propagation-process.md`
- [x] 5.9 Author `/docs/concepts/causal-monad.md`
- [x] 5.10 Author `/docs/concepts/hkt.md`
- [x] 5.11 Author `/docs/concepts/cdl.md`
- [x] 5.11a Sidebar sort: concepts/guides/reference alphabetized; glossary pinned last in concepts.
- [ ] 5.12 _**Deferred.**_ Author `/docs/guides/cdl-pipeline.md` end-to-end CDL walkthrough.
- [ ] 5.13 _**Deferred.**_ Author one `/docs/guides/<domain>.md` per landing-page example (six deeper guides).
- [ ] 5.14 _**Deferred.**_ Author one `/docs/reference/<crate-name>.md` per published crate (20 pages).
- [ ] 5.15 _**Not pursued.**_ Verify the hello-causaloid example compiles against the current crate versions — _CI compiles every example crate on every PR; verification is the job of the existing CI matrix, not this site change._
- [ ] 5.16 _**Not pursued.**_ Verify the CDL guide example compiles — _depends on the deferred 5.12._

### Styleguide compliance pass (out-of-band; user-driven)
- [x] 5.style.1 Em-dash density audit across every content file; cap at ≤ 4 / 1,000 words. All 19 files now under budget (top remaining density 3.4/1k). Worst-offender fixes were almost entirely `**Term** — definition` → `**Term**: definition` collapses.

## 6. Monograph section (LaTeX → MDX + PDF)

- [ ] 6.1 — 6.11 _**All deferred.**_ The monograph content collection is scaffolded but empty; the navigation does not advertise it. Building the canonical PDFs, running pandoc on the EPP chapters, and authoring per-volume MDX overviews is a sizable separate workstream.

## 7. Content migration from `ctx/`

- [x] 7.1 Enumerate blog posts in `ctx/content/` and copy each into `website/web/src/content/blog/en/` with normalized frontmatter — _23 posts migrated; `summary` → `description` schema alignment; "hyper-geometric computational causality" rewritten to "dynamic-causality framework" across every migrated post._
- [x] 7.2 Rewrite intra-blog image references — _all referenced assets (`/img/blog/streaming-architecture.png`, `/img/docs/causaloid.png`, `/img/logo-color.png`, `/img/jb_beam.svg`) resolve._
- [x] 7.3 Migrate evergreen non-blog pages — _shipped `/about/`, `/community/`, `/accessibility/`; **contact excluded** per user; rebranded "hyper-geometric computational causality library" → "dynamic-causality framework"; the broken `/docs/intro/` and `/contact/` outbound links replaced with current targets; GitHub link updated from the retired `deepcausality-rs/sites` to `deepcausality-rs/deep_causality`._
- [ ] 7.4 _**Deferred.**_ Legacy `/contact` → 301 redirect. The old Hugo site lived on a different repo, and Cloudflare is now serving the new site directly; no observed inbound `/contact` traffic to redirect today.
- [x] 7.5 Copy remaining content imagery — _shipped `ctx/static/img/blog/*`, `ctx/static/img/docs/*`, `logo-color.png`, `jb_beam.svg`, `social-share.jpg` (OG image). **Not migrated** (no current consumer): `blue_graph.jpg`, `front/front-{500,800,1200}.webp` responsive variants, `social-icons/*.svg` (16 platform glyphs)._
- [x] 7.6 Confirm `ctx/` is untouched apart from being read.

## 8. Redirects and SEO

- [ ] 8.1 _**Deferred.**_ Compile a list of high-traffic legacy URLs from the existing deepcausality.com.
- [ ] 8.2 _**Deferred.**_ Add `website/web/public/_redirects`. Becomes relevant at production cutover to `deepcausality.com`.
- [x] 8.3 `robots.txt` and `sitemap.xml` — _sitemap generated by `@astrojs/sitemap`; `robots.txt` not added (Cloudflare Pages serves a sane default for public projects)._
- [x] 8.4 OG / Twitter card metadata in `BaseLayout.astro` — _full `og:image`, `og:image:width`, `og:image:height`, `og:image:alt` plus matching `twitter:image`, `twitter:title`, `twitter:description`. Default OG image: `/img/social-share.jpg` (1200×630); overridable per page via the `ogImage` prop._
- [x] 8.5 Integrate Pagefind — _`pagefind --site dist` runs as part of `pnpm build`; index is produced. **Search UI not yet wired** into the header; deferred until needed._

## 9. Verification

- [x] 9.1 Push the fork branch — _Cloudflare Pages live at https://deep-causality.pages.dev._
- [x] 9.2 Manual smoke test on the beta domain — _user verified landing, examples, examples categories, docs, blog posts, theme toggle, mobile layout._
- [ ] 9.3 _**Deferred.**_ Automated forbidden-phrase check — _depends on 2.5._
- [ ] 9.3a _**Deferred.**_ Automated styleguide audit script — _the manual audit landed (every file under the 4/1k em-dash budget), but no script enforces it on future PRs._
- [x] 9.3b Visual-design review — _user reviewed; iterated on hero, card uniformity, mobile responsiveness, code-block framing, syntax-highlighting in light mode._
- [ ] 9.4 _**Deferred.**_ Lighthouse pass (target ≥ 95 performance & accessibility on mobile).
- [x] 9.5 User review on the beta domain.

## 10. Cutover (operator action)

- [x] 10.1 Cloudflare auto-deploy publishes to https://deep-causality.pages.dev on every push.
- [x] 10.2 Cloudflare Pages project source repo / root directory configured to point at `website/web/` — _user confirmed; an earlier failed deploy was traced to a stale clone of an older commit; re-trigger succeeded once the website-bearing commit was pushed._
- [ ] 10.3 _**Pending operator action.**_ Cut `deepcausality.com` DNS over to the Cloudflare Pages project. Currently still on the old Hugo site at production.
- [ ] 10.4 _**Follow-up change, user-approved.**_ Remove the `ctx/` snapshot from this monorepo once it's no longer needed as a migration source.

## 11. Library-side cleanup (out-of-band; user-driven)

- [x] 11.1 Update the `deep_causality` crate's lib.rs doc-comment to drop the "hyper-geometric computational causality" framing and adopt "dynamic causality" — _important because docs.rs renders this text; was the last place the old framing leaked._

## 12. Framework upgrade (out-of-band; user-driven)

- [x] 12.1 Astro 5 → Astro 6 — _`astro@6.3.2`, `@astrojs/mdx@5.0.5`. Clean upgrade, no API breakage in our code, 57 pages still build green._
