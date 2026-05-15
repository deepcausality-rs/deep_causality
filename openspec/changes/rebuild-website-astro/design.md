## Context

The current deepcausality.com is a Hugo site that lives in a **separate repository** (not in this monorepo). `ctx/` in this repo is a one-time snapshot of its Markdown content and images, copied here purely as a migration source. Two distinct problems compound:

1. **Technical**: Hugo's theming and shortcode layer have become an active maintenance drag. Routine content updates require theme spelunking; the build pipeline is brittle.
2. **Editorial**: The site leads with the framing "hypergeometric computational causality" and an extensive philosophical preamble. Engineers — the primary acquisition target — bounce before reaching anything actionable.

The project itself is a 20-crate Rust monorepo for computational causality. Its real-world surface — `Causaloid`, `Context` (hypergraph), `Effect Ethos`, dynamic context, mutable causal rules — is fundamentally about **dynamic causality**. The hypergeometric framing was an early formal scaffold, not the product's actual selling point.

A monograph rooted at `papers/src/EPP/` is the new canonical conceptual source. It is authored as **LaTeX**, not Markdown: six preprint volumes (`Preprint_EPP`, `Preprint_EPP_Metaphysics`, `Preprint_EPP_Formalization`, `Preprint_EPP_Ontology`, `Preprint_EPP_Epistemology`, `Preprint_EPP_Teleology`) plus shared `epp_chapters/`, `epp_appendices/`, `epp_bib/`, `styles/`, and `shared/`. Conversion to web-readable MDX is its own non-trivial sub-task. Existing Hugo docs are stale and cannot be trusted as a migration source for the doc tree — only for the blog and a handful of evergreen pages.

Constraints from `AGENTS.md`:
- No git commits without user approval.
- No file/folder deletion without user approval.
- Static dispatch / minimal-diff sensibility (applies to the Rust crates, not the site, but informs the no-cruft posture).

## Goals / Non-Goals

**Goals:**
- Stand up a new Astro site at `website/web/` deployable to Cloudflare Pages.
- Rebrand the public framing to "dynamic causality" consistently across the site.
- Landing page leads with six code examples spanning distinct engineering fields; each links to a dedicated detail page. Philosophical material moves below the fold and to docs.
- Newly authored docs sourced from the monograph + crate APIs replace the legacy Hugo docs.
- Migrate the blog and a curated set of static pages/assets from `ctx/`.
- Scaffold Astro i18n now (English at launch) so locales are additive later.
- Reduce maintenance surface: Markdown-first content, conventional Astro layout, no theme layer to fight.

**Non-Goals:**
- Translating any content to non-English locales in this change.
- Rewriting or restructuring the Rust crates.
- Building a CMS, comments system, or any dynamic backend.
- Search infrastructure beyond client-side filtering (e.g., Pagefind) if time allows; full-text search service is out of scope.
- Auto-generating API reference from `cargo doc`. Hand-written docs only in this round; `docs.rs` remains the API reference.
- Deleting `ctx/` in this change. Retirement is a separate user-approved step.

## Decisions

### D1. Framework: Astro
- **Choice**: Astro 4.x with the Markdown/MDX content collections API.
- **Why**: Native Markdown + frontmatter, first-class i18n routing, islands architecture keeps JS payload near zero for a marketing/docs site, deploys cleanly to Cloudflare Pages.
- **Alternatives considered**: Next.js (heavier, React-mandated, overkill for static-first content), Docusaurus (docs-shaped, weaker for marketing landing), Zola (Rust-native but smaller ecosystem, weaker i18n + MDX story), staying with Hugo (rejected — that's the problem).

### D2. Hosting: Cloudflare Pages
- **Choice**: Cloudflare Pages with the Astro `@astrojs/cloudflare` adapter set to static output for launch.
- **Why**: Free tier sufficient for traffic; Workers available later for any dynamic edge logic; user has prior Cloudflare familiarity.
- **Alternatives considered**: Vercel/Netlify (fine but no reason to switch ecosystems), self-hosted (unnecessary ops burden).

### D3. Repo location: `website/web/`
- **Why**: Leaves room under `website/` for sibling artifacts later (e.g., `website/og-image-generator/`) without polluting the root.
- The Astro project owns its own `package.json`, `pnpm-lock.yaml`, `tsconfig.json`, and CI workflow.

### D4. Content authoring: Markdown + MDX via content collections
- Docs and blog use Astro content collections with typed frontmatter schemas (`src/content/config.ts`).
- Code-example detail pages use MDX so they can embed reusable components (e.g., a syntax-highlighted Rust block with a "Run on Rust Playground" footer).

### D5. i18n strategy
- Astro's built-in i18n routing under `src/pages/[locale]/`.
- Default locale: `en`. `defaultLocale` configured with `routing: { prefixDefaultLocale: false }` so English URLs stay clean (`/docs/...`, not `/en/docs/...`).
- Content collections keyed by `{locale}/{slug}` so adding `de`, `ja`, etc. is purely additive.

### D6. Rebrand glossary (binding for all copy)
- **Old → New**:
  - "hypergeometric computational causality" → "dynamic causality"
  - "computational causality library" → "dynamic causality framework" (when used in marketing copy; the crate description in `AGENTS.md` may remain technical)
  - Tagline (confirmed): "Dynamic causality for advanced systems."
- Technical terms that stay: Causaloid, Context, Effect Ethos, Causal Reasoning, Effect Propagation Process.
- A short glossary page at `/docs/concepts/glossary` codifies this so future contributors don't drift.

### D7. Landing page structure
Above the fold:
1. Hero: logo + rebranded tagline + two CTAs ("Read the docs", "View on GitHub").
2. Six code-example cards in a 3×2 grid. Each card: domain icon, one-line problem statement, ~15-line Rust snippet, link to detail page.

Below the fold:
3. "What is dynamic causality?" — 3-paragraph plain-language explainer.
4. Pillars: Causaloid, Context, Effect Ethos — one card each, linking to docs.
5. Footer.

### D8. Six code-example domain selection (initial slate)
Chosen to span fields engineers recognize and to exercise different parts of the library:

1. **Quant finance / trading** — multi-stage causal rule firing on a price stream (uses `deep_causality` + sliding window from `deep_causality_data_structures`).
2. **Robotics / control** — context-aware effect propagation with a dynamic spacetime context (uses `Context` hypergraph + `deep_causality_topology`).
3. **Observability / SRE** — causal root-cause chain on log/metric events (uses `Causaloid` composition).
4. **Bioinformatics / signal processing** — MRMR feature selection from `deep_causality_algorithms`.
5. **Physics simulation** — effect propagation under changing constraints (uses `deep_causality_physics` + `deep_causality_multivector`).
6. **Policy / compliance** — Effect Ethos verifying operational rules at runtime (uses `deep_causality_ethos`).

Each detail page: longer code, runnable instructions, link to the corresponding crate(s) and to the relevant monograph chapter.

### D9. Documentation tree (new, not migrated)
Source of truth = monograph in `papers/` + current crate state.
```
/docs/
  getting-started/       (install, hello-causaloid, hello-context)
  concepts/              (dynamic-causality, causaloid, context, effect-ethos,
                          effect-propagation-process, causal-monad, hkt, glossary)
  guides/                (one per code-example domain, deepened)
  reference/             (per-crate overview pages — what each of the 20 crates does
                          and when to reach for it; deep API stays on docs.rs)
  monograph/             (the six EPP preprint volumes converted from LaTeX to MDX,
                          lightly re-edited for web)
```

**Concept-page note**: Three pieces the initial slate missed are first-class and must be covered:
- **Causal Monad** — implemented as `deep_causality_core::types::causal_monad`. The functional core of how effects compose and propagate.
- **HKT (Higher-Kinded Types)** — implemented in `deep_causality_haft` and used inside `deep_causality_core` (`types/causal_effect_propagation_process/hkt.rs`). The abstraction layer that lets the framework express the causal monad and effect propagation generically.
- **CDL (Causal Discovery Language)** — the DSL in `deep_causality_discovery` for going from raw observational data to actionable causal insights via a typestate-builder pipeline (config → data load → feature selection → discovery → analysis). CDL gets both a concept page (`/docs/concepts/cdl`) introducing the DSL and the typestate workflow, and a dedicated end-to-end guide page (`/docs/guides/cdl-pipeline`) walking through the example in `deep_causality_discovery/examples/main.rs`.

**Visual design — binding for all UI work on this site**: All UI, layout, typography, spacing, color, shadow, and component-architecture decisions MUST be made through the `design-taste-frontend` skill (and, where appropriate, the `high-end-visual-design` and `redesign-existing-projects` skills). This applies to: the landing-page hero and code-example grid, the docs layout, the example detail pages, the global header/footer, the search UI, and any reusable components. The goal is to override default LLM design biases — no generic Tailwind-card aesthetic, no centered-text-on-gradient hero, no rounded-2xl-shadow-md defaults. The skill's metric-based rules (component architecture, CSS hardware acceleration, balanced design engineering) are binding constraints on PR review for this change.

**Writing style — binding for all new and edited copy on this site**: All prose authored or edited under this change MUST follow `docs/writing_guides/Ai Styleguide.md`. Concrete operating rules derived from that guide:
- Em dashes: ≤ 4 per 1,000 words. Prefer periods or commas.
- Use semicolons where they fit; their total absence is itself a tell.
- Vary sentence length deliberately. Target a span from ~3 to ~35 words within any given page; do not cluster in the 12–18-word band.
- Avoid AI-tell phrases entirely: "delve into", "shed light on", "game-changer", "unlock the potential", "not only … but also".
- Avoid "Additionally," / "Furthermore," as paragraph openers; keep their ratio below 0.4 per paragraph count.
- Filler words ("very", "really"): keep total below 2% of word count.
- Do not cycle "crucial / vital / essential / significant" as synonyms for "important". Use "important", "key", or "major" sparingly and stop.
- Break subject-verb-object monotony; mix structures.

These rules apply to landing-page copy, concept pages, guides, reference pages, blog rewrites, and monograph overview pages. The monograph LaTeX source itself is exempt — it stands as the authors wrote it.

**Source material from `ctx/docs/`**: The high-level intro files in `ctx/docs/` (`INTRO.md`, `CORE.md`, `DEEP_DIVE.md`, `HAFT.md`, `ETHOS.md`, `DISCOVERY.md`, `PHYSICS.md`, `TOPOLOGY.md`, `TENSOR.md`, `UNIFORM_MATH.md`) are usable raw material for the new docs and reference pages — they should be read, rewritten under the styleguide and the rebrand, and folded into the appropriate `/docs/concepts/` and `/docs/reference/` pages. They are not migrated verbatim, and their structure is not preserved.

**LaTeX → MDX conversion**: The monograph is LaTeX. Options:
1. Pandoc `latex → markdown` per chapter, then hand-cleanup (math via KaTeX/MathJax in Astro).
2. Author web-native MDX summaries of each preprint, linking to the canonical PDFs for the full text.

Recommendation: hybrid. Publish the PDFs as canonical, plus an MDX overview per preprint (sourced via pandoc + cleanup) for in-site reading and SEO. This bounds the scope; the LaTeX source remains the source of truth.

### D10. Asset handling
- All static assets move to `website/web/public/`.
- `ctx/static/img/frontpage-art.webp` → `website/web/public/img/frontpage-art.webp` (called out explicitly because its current location is non-standard for the old Hugo tree).
- Favicons, OG images, logo variants live under `public/brand/`.

### D11. Build & deployment
- pnpm-managed Astro project within `website/web/` (independent from the Rust workspace).
- **No GitHub Actions workflow** is added by this change. Cloudflare Pages is already wired up: pushes to a fork branch auto-deploy to a beta domain; merges to `main` auto-deploy to production deepcausality.com. The only operator action at cutover is updating the Cloudflare Pages project's source repo / root directory to point at this monorepo's `website/web/` path.
- Build command: `pnpm install && pnpm build`. Output directory: `website/web/dist/`. Root directory in Cloudflare: `website/web`.

### D12. Migration source carve-out
Migrated from `ctx/`:
- Blog posts (`ctx/content/blog/**` → `website/web/src/content/blog/`), frontmatter normalized.
- Hero/brand imagery from `ctx/static/img/`.
- A small allowlist of evergreen pages (about, contact, license) — to be confirmed during task execution.

NOT migrated (replaced):
- All `ctx/content/docs/**` — superseded by the new docs tree.
- Theme files, layouts, partials.

## Risks / Trade-offs

- **Rebrand confusion** → Pin the glossary page early; do a global search to ensure no stray "hypergeometric" copy in launched pages. Mitigation: add a CI grep check that fails the build if forbidden phrases appear outside `/docs/monograph/` and the glossary's "former framing" callout.
- **Doc rewrite scope creep** — writing comprehensive new docs is the largest unknown. Mitigation: ship landing + six code-example detail pages + getting-started + concepts first; remaining doc sections can land iteratively post-launch.
- **Two sites live simultaneously** during cutover — risk of stale Hugo links indexed in search. Mitigation: 301 redirects configured in Cloudflare Pages `_redirects` for known old paths.
- **Cloudflare Pages limits** (file count, build minutes) — likely a non-issue at this scale; flagged for awareness.
- **i18n scaffolding without translations** is dead weight if no second locale ever lands. Trade-off accepted: scaffolding cost is small, refactor cost later is large.
- **MDX islands** can balloon JS bundles if used carelessly. Mitigation: code-example detail pages use static MDX only; no client-side islands unless justified.

## Migration Plan

1. Bootstrap Astro project at `website/web/` (D1, D3).
2. Implement base layout, i18n config, brand tokens, glossary page (D5, D6).
3. Build landing page with six code-example cards (D7) — placeholders for snippets where final code isn't ready.
4. Author the six detail pages (D8).
5. Author getting-started + core concepts docs from the monograph (D9).
6. Migrate blog and allowlisted assets from `ctx/` (D10, D12).
7. Push the fork branch; Cloudflare's existing auto-deploy publishes the build to the beta domain (D11).
8. User reviews on the beta domain.
9. Merge fork branch into `main`; Cloudflare auto-deploys to production deepcausality.com. Operator updates the Cloudflare Pages project's source repo / root directory to `website/web/` as part of cutover.
10. (Separate, user-approved change) Remove the `ctx/` snapshot from this monorepo once it's no longer needed as a migration source. The external Hugo repo is unaffected.

Rollback: DNS cutover is reversible by repointing back to the existing Hugo deployment. Both deployments coexist until cutover is confirmed stable.

## Open Questions

- ~~Do we want client-side search (Pagefind) at launch, or defer?~~ **Decided: Pagefind at build time, included at launch.**
- ~~Final wording of the tagline~~ **Decided: "Dynamic causality for advanced systems."**
- Which evergreen pages from `ctx/content/` (beyond blog) actually migrate? Decide during task execution by reviewing the tree.
- Locale plan beyond English — which is the likely second locale (drives any framework-level choices that should be made now vs. later)?
