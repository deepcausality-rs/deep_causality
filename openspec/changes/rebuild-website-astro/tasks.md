## 1. Astro project bootstrap

- [x] 1.1 Create `website/web/` directory and initialize an Astro 4.x project with TypeScript, MDX, and the Cloudflare adapter (static output) — _used Astro 5.18 (current stable); static output, no Cloudflare adapter needed since Pages serves `dist/` directly. Adapter can be added later if SSR is wanted._
- [x] 1.2 Configure pnpm; add `package.json`, `pnpm-lock.yaml`, `.npmrc`, and a `.gitignore` for `node_modules` and `dist` — _plus `pnpm-workspace.yaml` `allowBuilds` for esbuild/sharp under pnpm 11, and root `.gitignore` updated._
- [x] 1.3 Verify the Rust Cargo workspace is unaffected (`cargo metadata` from repo root still resolves; no member added) — _30 workspace members, none under `website/`._
- [x] 1.4 Configure `astro.config.mjs` with `i18n: { defaultLocale: 'en', locales: ['en'], routing: { prefixDefaultLocale: false } }`
- [x] 1.5 Add brand-token CSS module (colors, typography, spacing) and a global stylesheet — _bootstrap-only tokens at `src/styles/tokens.css`, marked as placeholder until task 2.0 produces the real design system._
- [x] 1.6 Create `BaseLayout.astro` with shared header, footer, and meta defaults using rebranded copy
- [x] 1.7 Define content collection schemas in `src/content.config.ts` for: `blog`, `docs`, `examples`, `monograph`
- [x] 1.8 Verify `pnpm install && pnpm build` produces a static site in `website/web/dist/` — _builds clean, sitemap and Pagefind index generated._

## 2. Brand identity & visual design

- [x] 2.0 Invoke the `design-taste-frontend` skill to produce the design direction: typography scale, color system, spacing scale, shadow language, motion rules, and component-architecture conventions. Output committed as `website/web/DESIGN.md` and referenced by every page implementation below. — _Done. Dark + light themes (dark default), accent calibrated to hero art, anti-patterns list. Canonical tokens in `src/styles/tokens.css`. Pre-paint theme script lives in BaseLayout._
- [ ] 2.1 Place logo variants and favicons under `website/web/public/brand/`
- [ ] 2.2 Move the hero art from `ctx/static/img/frontpage-art.webp` to `website/web/public/img/frontpage-art.webp` (copy; do not delete source per Golden Rule)
- [x] 2.3 Author `/docs/concepts/glossary` defining: dynamic causality, Causaloid, Context, Effect Ethos, Causal Reasoning, Effect Propagation Process — plus a "former framing" entry addressing the prior "hypergeometric computational causality" label — _written at `src/content/docs/concepts/glossary.md`; covers all required terms plus Causal Monad, HKT, CDL, Teloid, Contextoid, plus former-framing note and three-modalities table._
- [ ] 2.4 Audit all hero, nav, and meta copy to use the "dynamic causality" framing
- [ ] 2.5 Add a build-time check (script invoked from `pnpm build`) that greps `dist/**/*.html` for "hypergeometric" and fails on any hit outside `/docs/monograph/` and `/docs/concepts/glossary/`, and for the AI Styleguide banned phrases ("delve into", "shed light on", "game-changer", "unlock the potential", "not only … but also") and fails on any hit anywhere
- [x] 2.6 Confirm with user: final tagline wording (D6 candidate or alternative) — _confirmed: "Dynamic causality for advanced systems."_

## 3. Landing page

- [x] 3.1 Implement the hero section: logo, rebranded tagline, "Read the docs" CTA → `/docs/getting-started`, "View on GitHub" CTA → repo URL — _asymmetric split (7/12 + 5/12); hero art frame on the right at desktop, stacks above text below 900px._
- [x] 3.2 Implement the six-card code-example grid component — _3×2 grid with middle-card stagger above 1200px, 2-col at ≤1080, 1-col at ≤720._
- [x] 3.3 Write or assemble the six Rust snippets (10–20 lines each) for: quant finance / trading, robotics / control, observability / SRE, bioinformatics / signal processing, physics simulation, policy / compliance — _illustrative snippets in `src/components/home/examples.ts`; compilable end-to-end versions land on detail pages._
- [x] 3.4 Add syntax highlighting (Shiki via Astro) for Rust code blocks — _`shikiConfig` in `astro.config.mjs`, `github-dark` theme, rendered via the built-in `<Code>` component._
- [x] 3.5 Implement the below-the-fold "What is dynamic causality?" explainer (3 short paragraphs) — _prose-measure constrained, no decorative imagery, anchored on the rebrand._
- [x] 3.6 Implement the three-pillar section linking to `/docs/concepts/causaloid`, `/docs/concepts/context`, `/docs/concepts/effect-ethos` — _SVG connector path across the row at desktop; stacks with vertical hairlines below 900px. Not a 3-equal-cards row._
- [ ] 3.7 Verify at 1440×900 that above-the-fold contains only hero + first row of cards (no philosophical copy) — _layout sized to satisfy this; verification needs a browser/headless run on the user side._

### Vendored fonts (out-of-band improvement)
- [x] 3.x Vendor `Geist Variable` and `JetBrains Mono Variable` woff2 files into `public/fonts/`; add `@font-face` rules in `src/styles/fonts.css`; preload primary files in `BaseLayout.astro`; remove `@fontsource` packages so no node_modules indirection at runtime — _per user feedback: avoid CDN/render-blocking font loads._

## 4. Six code-example detail pages

- [x] 4.1 Create the `examples` collection route `/examples/[slug]` driven by MDX files in `src/content/examples/en/` — _route at `src/pages/examples/[...slug].astro`, plus `/examples/` index._
- [x] 4.2 Author `examples/en/quant-finance.mdx` with expanded code, walkthrough, run instructions, related crates, further reading
- [x] 4.3 Author `examples/en/robotics-control.mdx`
- [x] 4.4 Author `examples/en/observability-sre.mdx`
- [x] 4.5 Author `examples/en/bioinformatics-signal.mdx`
- [x] 4.6 Author `examples/en/physics-simulation.mdx`
- [x] 4.7 Author `examples/en/policy-compliance.mdx`
- [x] 4.8 Verify each landing-page card links to its corresponding detail page and all six pages render — _all six slugs build; landing-page card hrefs match the generated routes._

## 5. Documentation tree (newly authored)

- [x] 5.0 Read `docs/writing_guides/Ai Styleguide.md` and `ctx/docs/*.md` before authoring any of the pages below; treat the styleguide as binding for every prose page in §3, §4, §5, §6, and §7 — _styleguide internalized; concept pages also drew from monograph (`papers/src/EPP/`) and `deep_causality_core` source via Explore-agent surveys before authoring._
- [x] 5.1 Author `/docs/getting-started/install.md`
- [x] 5.2 Author `/docs/getting-started/hello-causaloid.md` with a copy-pasteable end-to-end runnable example
- [x] 5.3 Author `/docs/getting-started/hello-context.md`
- [x] 5.4 Author `/docs/concepts/dynamic-causality.md` (the umbrella concept page) — _grounded in the monograph's three-modality framing (static/dynamic/emergent) and the `m₂ = m₁ >>= f` axiom from `causality_as_epp.tex`._
- [x] 5.5 Author `/docs/concepts/causaloid.md` — _quotes the actual struct from `deep_causality/src/types/causal_types/causaloid/mod.rs`; covers Singleton/Collection/Graph isomorphism._
- [x] 5.6 Author `/docs/concepts/context.md` — _quotes the actual struct from `deep_causality/src/types/context_types/context_graph/mod.rs`; covers the five Contextoid payload kinds._
- [x] 5.7 Author `/docs/concepts/effect-ethos.md` — _grounded in `teleology.tex` and the `deep_causality_ethos` crate; covers Teloid + DDIC + Lex Posterior/Specialis/Superior._
- [x] 5.8 Author `/docs/concepts/effect-propagation-process.md` — _shows the literal `CausalEffectPropagationProcess<V,S,C,E,L>` struct and the EffectValue enum._
- [x] 5.9 Author `/docs/concepts/causal-monad.md` sourced from `deep_causality_core/src/types/causal_monad/` — _includes the actual `pure`/`bind` signatures and the monad-law statement._
- [x] 5.10 Author `/docs/concepts/hkt.md` sourced from `deep_causality_haft/src/hkt/` and `deep_causality_core/src/types/causal_effect_propagation_process/hkt.rs` — _explains the witness pattern and HKT5/HKT3 traits._
- [x] 5.11 Author `/docs/concepts/cdl.md` introducing the Causal Discovery Language, the typestate-builder pipeline (config → load → feature selection → discovery → analysis), and when to reach for it, sourced from `deep_causality_discovery/README.md` and `deep_causality_discovery/src/`
- [ ] 5.12 Author `/docs/guides/cdl-pipeline.md` — end-to-end CDL walkthrough based on `deep_causality_discovery/examples/main.rs`, with copy-pasteable code blocks and run instructions
- [ ] 5.13 Author one `/docs/guides/<domain>.md` per landing-page example (six pages, deeper than the example detail pages)
- [ ] 5.14 Author one `/docs/reference/<crate-name>.md` per published crate (20 pages), each with summary, primary types/traits, and a link to docs.rs
- [ ] 5.15 Verify the hello-causaloid example compiles against the current crate versions in this monorepo
- [ ] 5.16 Verify the CDL guide example compiles and runs against the current `deep_causality_discovery` version

## 6. Monograph section (LaTeX → MDX + PDF)

- [ ] 6.1 Build canonical PDFs from `papers/src/EPP/Preprint_EPP*` LaTeX sources and place them under `website/web/public/monograph/`
- [ ] 6.2 Run pandoc `latex → markdown` on the chapter files in `papers/src/EPP/epp_chapters/` and `papers/src/EPP/epp_appendices/` to produce raw MDX drafts; configure KaTeX/MathJax in Astro for math rendering
- [ ] 6.3 Author MDX overview page for `Preprint_EPP` at `/docs/monograph/epp/`
- [ ] 6.4 Author MDX overview page for `Preprint_EPP_Metaphysics` at `/docs/monograph/metaphysics/`
- [ ] 6.5 Author MDX overview page for `Preprint_EPP_Formalization` at `/docs/monograph/formalization/`
- [ ] 6.6 Author MDX overview page for `Preprint_EPP_Ontology` at `/docs/monograph/ontology/`
- [ ] 6.7 Author MDX overview page for `Preprint_EPP_Epistemology` at `/docs/monograph/epistemology/`
- [ ] 6.8 Author MDX overview page for `Preprint_EPP_Teleology` at `/docs/monograph/teleology/`
- [ ] 6.9 Each overview page links to its canonical PDF in `/monograph/`
- [ ] 6.10 Add a `/docs/monograph/` index page that lists all six volumes and explains the LaTeX-is-canonical relationship
- [ ] 6.11 Add bidirectional links between concept pages and their corresponding monograph volumes / chapters

## 7. Content migration from `ctx/`

- [ ] 7.1 Enumerate blog posts in `ctx/content/` and copy each into `website/web/src/content/blog/en/` with normalized frontmatter
- [ ] 7.2 Rewrite intra-blog image references to use the new `/img/` paths
- [ ] 7.3 Migrate evergreen non-blog pages from `ctx/content/` (about, license, and other static pages). **Exclude contact** — no contact form on the new site.
- [ ] 7.4 If the legacy site has a `/contact` route, add a 301 redirect to an appropriate destination (e.g., GitHub issues or the about page) in `_redirects`
- [ ] 7.5 Copy remaining content imagery from `ctx/static/img/` to `website/web/public/img/`
- [ ] 7.6 Confirm `ctx/` is untouched apart from being read; do not delete (Golden Rule)

## 8. Redirects and SEO

- [ ] 8.1 Compile a list of high-traffic legacy URLs from the existing deepcausality.com
- [ ] 8.2 Add `website/web/public/_redirects` mapping legacy paths to their new equivalents (301s)
- [ ] 8.3 Add `robots.txt` and a generated `sitemap.xml` (via `@astrojs/sitemap`)
- [ ] 8.4 Add OG / Twitter card metadata to `BaseLayout.astro`
- [ ] 8.5 Integrate Pagefind: run `pagefind --site dist` as a post-build step in `pnpm build`; add a search UI component wired to the Pagefind index; include search in the global header

## 9. Verification

- [ ] 9.1 Push the fork branch and confirm Cloudflare's existing auto-deploy publishes to the beta domain
- [ ] 9.2 Manual smoke test on the beta domain: landing page above-the-fold, all six example pages, docs index, monograph index, a sample blog post
- [ ] 9.3 Run the build-time forbidden-phrase check and confirm zero violations (covers "hypergeometric" plus the AI Styleguide banned phrases: "delve into", "shed light on", "game-changer", "unlock the potential", "not only … but also")
- [ ] 9.3a Run a styleguide audit script: em-dash density ≤ 4/1k words per page, filler-word ratio < 2%, "Additionally|Furthermore" paragraph-opener ratio < 0.4; fail loudly on any violation
- [ ] 9.3b Visual-design review pass against the `design-taste-frontend` skill output: no generic-default patterns shipped, component architecture matches `DESIGN.md`, motion/perf rules followed
- [ ] 9.4 Lighthouse pass on landing page (target: performance ≥ 95 mobile, accessibility ≥ 95)
- [ ] 9.5 User review on the beta domain

## 10. Cutover (operator action, not in this change's diff)

- [ ] 10.1 Merge fork branch into `main`; Cloudflare auto-deploys to production
- [ ] 10.2 Operator updates the Cloudflare Pages project source repo / root directory to point at this monorepo's `website/web/` if not already configured
- [ ] 10.3 Confirm deepcausality.com serves the new site
- [ ] 10.4 (Follow-up change, user-approved) Remove `ctx/` snapshot from this monorepo
