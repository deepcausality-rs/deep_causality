# Proposal: Migrate documentation to Starlight on a dedicated subdomain

**Status:** Draft for review. No implementation has started.
**Author:** drafted with Claude
**Date:** 2026-05-28

## 1. Context and motivation

The documentation is outgrowing the hand-built Astro site in `website/web`, and it is growing fast. Today the docs live inside the main site as content collections (`src/content/docs/{getting-started,overview,concepts}`, plus `examples` and `blog`), rendered through bespoke components (`src/components/docs/DocsSidebar.astro`, `src/pages/docs/[...slug].astro`). Every docs feature (sidebar, search, prev/next, table of contents) is maintained by hand.

Current inventory (`website/web/src/content`):

- `docs/getting-started`: 6 pages
- `docs/overview`: 6 pages (now includes the new Literature page)
- `docs/concepts`: 13 pages
- `examples/en`: 20 pages
- `blog/en`: 29 pages

[Starlight](https://starlight.astro.build/) is Astro's official documentation framework. It is built on the same Astro the site already uses, so the Markdown/MDX content and frontmatter conventions are close to what we have. Starlight gives us, out of the box: a maintained sidebar and nav, full-text search (Pagefind, which we already invoke in the build), automatic prev/next, table of contents, last-updated, edit links, and i18n scaffolding. Moving docs onto it removes a growing maintenance burden and unlocks documentation features we would otherwise hand-roll.

This proposal also pulls in three capabilities the user asked for:

1. Proper code highlighting (Starlight ships Expressive Code: line highlighting, diffs, titles, frames).
2. A graph view via the [Starlight Obsidian theme](https://fevol.github.io/starlight-theme-obsidian/getting-started/) (`starlight-theme-obsidian`), which renders a backlinks graph of the docs.
3. A build-time render of the whole documentation into a single PDF via [`starlight-to-pdf`](https://github.com/Linkerin/starlight-to-pdf).

## 2. Goals and non-goals

**Goals**

- Move the long-form documentation (concepts, deep getting-started, deep overview, and likely examples) into a dedicated Starlight app under `website/docs`.
- Serve it from a dedicated origin, `https://docs.deepcausality.com`, deployed by its own Cloudflare Worker, so a docs change rebuilds and redeploys only the docs, leaving `www.deepcausality.com` untouched.
- Keep a short getting-started and a short overview on the marketing site (`website/web`), plus a new "Documentation" landing page on the marketing site that points outward to the Starlight docs and to the Rust API docs on docs.rs.
- Add code highlighting, the Obsidian graph view, and a single-PDF export.
- Preserve SEO: no loss of existing `/docs/*` ranking, clean crawler behavior across the two origins.

**Non-goals (for the first cut)**

- Migrating the blog. The blog is news/marketing and stays on `www`.
- SSR or dynamic features. Both sites stay fully static.
- Redesigning content. This is a platform migration, not a rewrite. The CausalMonad-trait reframe already landed separately.

## 3. Proposed architecture

Two independent static Astro builds in one repo, two Cloudflare Workers, two hostnames under one Cloudflare zone (`deepcausality.com`).

```
website/
  web/      → existing marketing site      → Worker "deepcausality-prod"  → www.deepcausality.com
  docs/     → NEW Starlight documentation   → Worker "deepcausality-docs"  → docs.deepcausality.com
```

- `website/docs` is a fresh Starlight project (`npm create astro@latest -- --template starlight` into that folder), with its own `package.json`, `astro.config.mjs`, and `wrangler.toml` (`name = "deepcausality-docs"`, `[assets] directory = "./dist"`), mirroring the pattern in `website/web/wrangler.toml`.
- `site:` in the docs Astro config is `https://docs.deepcausality.com`, so the sitemap and canonical URLs are correct for the new origin.
- Each Worker has its own custom domain bound in the Cloudflare dashboard. `www` already routes to `deepcausality-prod`; `docs` gets a CNAME/route to `deepcausality-docs`. Independent deploys: `wrangler deploy` in `website/docs` touches only the docs Worker.
- CI: split the deploy so a change under `website/docs/**` triggers only the docs build/deploy, and a change under `website/web/**` triggers only the marketing build/deploy (path filters). This is the "only the changed part rebuilds" property the user wants.

### Content split

| Content | Destination | Notes |
| --- | --- | --- |
| `docs/concepts/*` (13) | Starlight (`website/docs`) | The bulk of the long-form docs. |
| `docs/getting-started` deep pages | Starlight | Full hello-* walkthroughs. |
| `docs/overview` deep pages (incl. Literature, Innovations, Problem) | Starlight | Long-form overview. |
| `examples/en/*` (20) | **stays on `www`** | Decided: examples remain on the marketing site with their current components and the Examples dropdown. |
| Short getting-started + short overview | stays on `www` | New, condensed marketing-flavored summaries. |
| New "Documentation" landing page | `www` | Links to docs.deepcausality.com and docs.rs. |
| `blog/en/*` (29) | stays on `www` | Out of scope. |

Because examples stay on `www`, any internal links from migrated docs into `/examples/*` become **cross-origin** links to `https://www.deepcausality.com/examples/*` (absolute URLs), and vice versa.

The marketing site keeps an entry point: a `/docs` (or `/documentation`) page on `www` that frames the docs and links out to `https://docs.deepcausality.com` (Starlight) and to the Rust API reference on [docs.rs](https://docs.rs/deep_causality). The short overview and short getting-started remain on `www` for the first-touch visitor; the deep versions live in Starlight.

## 4. Starlight features to enable

- **Shared visual identity (decided).** The Starlight docs adopt the marketing site's look as closely as possible: the same color tokens, fonts, and logo, so `www` and `docs` read as one product. This is done through Starlight's theming surface (custom CSS, `components` overrides for the header/footer where needed, and matching the design tokens in `website/web` `DESIGN.md` / `global.css`). The `starlight-theme-obsidian` base is then restyled to those tokens rather than left stock.
- **Code highlighting.** Starlight's Expressive Code is on by default. We carry over the dual light/dark theme intent from the current Shiki config (`github-light` / `github-dark` in `website/web/astro.config.mjs`).
- **Graph view.** Add `starlight-theme-obsidian` for the backlinks graph and Obsidian-style wiki links. This rewards the `[[name]]`-style cross-linking the docs already lean toward.
- **Single-PDF export (decided: local build).** Add `starlight-to-pdf`, but run it from a **local build script**, not in CI. It drives a headless browser (Puppeteer), and the Cloudflare build environment does not provide one. The workflow: run the script locally to regenerate the PDF before pushing, and commit the PDF so the docs and the derived PDF stay consistent in the repo. The Cloudflare docs build then just serves the committed PDF as a static asset; it never launches a browser.
- **Mermaid.** The marketing site uses `astro-mermaid`. Starlight has its own mermaid handling (e.g. `astro-mermaid` or a Starlight-specific approach); confirm parity for any diagrams that move.

## 5. SEO and the Google crawler (the key question)

**Short answer: a subdomain is treated by Google as a separate site for crawling and indexing. You should give `docs.deepcausality.com` its own sitemap and verify it in Search Console. The crawler will not perfectly "figure it out" on its own, and during migration you must redirect the old paths.**

Specifics and recommendations:

1. **Search Console.** Use a **Domain property** for `deepcausality.com` (DNS-verified). A Domain property covers every subdomain (`www`, `docs`, apex) under one property, so both origins report together. If you prefer URL-prefix properties, add one for `https://docs.deepcausality.com` explicitly. Either way, **submit the docs sitemap** (`https://docs.deepcausality.com/sitemap-index.xml`) to Search Console; do not assume discovery.
2. **Per-origin sitemap + robots.txt.** The docs Worker serves its own `sitemap-index.xml` (via `@astrojs/sitemap` with `site: https://docs.deepcausality.com`) and its own `robots.txt` that points to that sitemap. The `www` sitemap drops the `/docs/*` entries once they move.
3. **Redirects preserve ranking (critical).** `/docs/*` currently lives on `www.deepcausality.com` and is indexed there. When docs move to the subdomain, add **301 redirects** from `www.deepcausality.com/docs/<path>` to `https://docs.deepcausality.com/<path>` (Cloudflare `_redirects` / Bulk Redirects, or a rule on the `www` Worker). This passes existing link equity and prevents duplicate content. Without it, the migration loses the ranking the current docs have earned.
4. **Canonical URLs.** Each docs page sets its canonical to the `docs.` origin (Starlight + `site:` handles this). This removes ambiguity if any old `www/docs` URL lingers.
5. **Cross-linking.** Link from `www` (the new Documentation landing page, the nav, the footer) to `docs.deepcausality.com`, and link back from docs to `www`. Internal links are the main discovery path Google uses across the two origins; the redirects plus cross-links make discovery reliable.
6. **Two virtual origins on one CDN is fine.** Two Workers serving two hostnames under the same registrable domain is a normal, well-understood setup. Google does not penalize it. The work is in sitemaps, redirects, and Search Console registration, not in the origin topology itself.

Net: the crawler does not automatically treat the new subdomain as part of the `www` property; register it (a Domain property is the least-effort path), submit its sitemap, and 301 the old `/docs` paths.

## 6. Risks and things to verify

- **Astro 6 vs Starlight compatibility.** The marketing site is on Astro `^6.3.5`. Confirm the current Starlight release (and `starlight-theme-obsidian`, `starlight-to-pdf`) supports Astro 6 before committing. The docs app has its own `package.json`, so its Astro version can differ from the marketing site if needed; they do not have to share a version.
- **`starlight-to-pdf` is a local step (decided).** It runs a headless browser, which the Cloudflare build env lacks, so it is not part of the Cloudflare build. It runs from a local script before push, and the resulting PDF is committed and served as a static asset. Risk reduces to "remember to regenerate before pushing"; a pre-push reminder or a check that flags a stale PDF would help.
- **Frontmatter mapping.** Current docs frontmatter uses `title`, `description`, `section`, `order`, `sectionLabel`. Starlight uses its own frontmatter (`title`, `description`, `sidebar.order`, `sidebar.label`, etc.) and derives the sidebar from directory structure or an explicit `sidebar` config. Migration is mostly mechanical but needs a mapping pass.
- **Internal link rewrites.** Links like `/docs/concepts/causal-monad/` become root-relative on the docs origin (`/concepts/causal-monad/` or `/guides/...` depending on the Starlight tree). A find/replace pass plus the 301s covers both in-docs links and inbound links from `www`.
- **Examples coupling.** The `examples/en/*` pages and the `/examples/*` routes use marketing-site components and the Examples dropdown. If examples move to Starlight, those components and routes need a decision (open question 4).
- **Search.** Pagefind is already in the `www` build script; Starlight bundles Pagefind, so docs search comes for free, but it indexes only the docs origin. Site-wide search across both origins is not automatic.
- **pnpm wiring (decided: standalone).** `website/docs` is standalone, not part of `website/web`'s pnpm workspace, so its dependencies, lockfile, and Astro version are independent. Any pnpm overrides it needs live in its own `pnpm-workspace.yaml` (per the project rule that overrides must live there, not in `package.json`).

## 7. Rough phased plan (for a later, approved change)

1. Scaffold `website/docs` as a Starlight app; wire `site`, sitemap, `wrangler.toml` (`deepcausality-docs`).
2. Add the Obsidian theme and Expressive Code; confirm dual-theme highlighting and the graph view render.
3. Migrate `concepts` first (the largest, most self-contained block), mapping frontmatter and rewriting internal links.
4. Migrate the deep getting-started and overview pages. Examples stay on `www`; rewrite docs↔examples links as cross-origin absolute URLs.
5. Add `starlight-to-pdf` as a local build script; verify the single-PDF output and commit it.
6. On `www`: add the short getting-started + short overview summaries and the Documentation landing page; remove the migrated long-form pages; drop `/docs/*` from the `www` sitemap.
7. Wire 301 redirects `www/docs/* → docs.deepcausality.com/*`.
8. Cloudflare: create the `deepcausality-docs` Worker, bind `docs.deepcausality.com`, split CI by path.
9. Search Console: confirm/add the Domain property, submit the docs sitemap.
10. Post-launch: watch Search Console coverage for both origins; verify redirects resolve and canonicals are correct.

## 8. Resolved decisions

All six open questions are now decided:

1. **Subdomain vs subdirectory → subdomain.** `docs.deepcausality.com`, its own Worker, independent rebuild. Chosen for operational independence (a docs change must not rebuild `www`), accepting the one-time SEO setup (sitemap + Search Console + 301s) as the price. Rationale recorded in §5.
2. **Search Console → Domain property for `deepcausality.com`.** A single DNS-verified Domain property covers `www`, `docs`, and the apex together. Submit the docs sitemap to it explicitly.
3. **Design system → match the marketing site.** The docs adopt the marketing site's visual identity as closely as possible (colors, fonts, logo) for cross-property consistency. See §4.
4. **Examples → stay on `www`.** Not migrated; they keep their current marketing-site components and the Examples dropdown. Cross-origin links handle docs↔examples references.
5. **PDF → local build script.** `starlight-to-pdf` runs locally (it needs a headless browser, which Cloudflare's build env lacks). Regenerate and commit the PDF before pushing so docs and PDF stay consistent; Cloudflare serves the committed PDF as a static asset. See §4.
6. **Monorepo wiring → standalone.** `website/docs` is its own self-contained build and setup, maintained independently of `website/web`. The documentation will update far more frequently than the marketing site as the project evolves, so the two are deliberately decoupled (separate `package.json`, lockfile, Astro version freedom, and deploy).

## 9. Next step

The architecture and all six decisions are settled. The next step is a formal change spec / implementation plan covering the phased plan in §7 against these decisions, for separate review before any code is written.
