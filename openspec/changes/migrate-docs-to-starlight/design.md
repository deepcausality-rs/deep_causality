## Context

The documentation lives inside the marketing site (`website/web`, Astro `^6.3.5`, fully static, deployed as Cloudflare Worker static assets at `www.deepcausality.com`). Docs are content collections (`src/content/docs/{getting-started,overview,concepts}`, plus `examples` and `blog`) rendered through bespoke components (`src/components/docs/DocsSidebar.astro`, `src/pages/docs/[...slug].astro`). Inventory: getting-started 6, overview 6, concepts 13, examples 20, blog 29.

Every docs affordance (sidebar, search, prev/next, TOC) is hand-maintained, and a docs edit rebuilds the whole marketing site. The docs grow faster than the marketing content and need their own platform and release cadence. Starlight is Astro's official docs framework on the same Astro foundation, so content and frontmatter map closely. The full reasoning and the six platform decisions are recorded in `openspec/notes/Starlight-Docs.md`; this design is the technical articulation of that note.

## Goals / Non-Goals

**Goals:**
- Stand up a standalone Starlight app in `website/docs`, independent of `website/web` (own deps, lockfile, Astro version, deploy).
- Serve docs from `docs.deepcausality.com` via a dedicated Cloudflare Worker; a docs change rebuilds only the docs.
- Migrate the long-form docs (concepts, deep getting-started, deep overview) with frontmatter mapping and internal-link rewrites.
- Match the marketing site's visual identity so the two origins read as one product.
- Add code highlighting, the Obsidian backlinks graph, and a single committed PDF.
- Preserve SEO: no loss of existing `/docs/*` ranking; clean crawler behavior across two origins.
- Keep a short getting-started and short overview on `www`, plus a Documentation landing page linking to the docs subdomain and docs.rs.

**Non-Goals:**
- Migrating the blog (stays on `www`).
- Migrating examples (stay on `www`).
- SSR or any dynamic features; both sites stay fully static.
- Rewriting documentation content. This is a platform migration; the CausalMonad-trait reframe already landed separately.
- Running the PDF render in CI.

## Decisions

**D1. Subdomain over subdirectory.** Docs live at `docs.deepcausality.com` on a separate Worker, not at `www.deepcausality.com/docs`.
- *Why:* operational independence is the primary goal; a separate Worker gives independent build/deploy with no proxy plumbing, and Starlight runs naturally at an origin root (no `base` path).
- *Alternative:* subdirectory consolidates SEO authority on one property with zero redirect work, but couples the builds. Regaining independent deploys under a subdirectory needs a stitching Worker (route `/docs/*` to a second Worker, Starlight `base: '/docs'`), trading SEO simplicity for deploy complexity. Rejected in favor of the simpler operational model; the SEO cost is a bounded one-time task (D4).

**D2. Standalone app, not a shared pnpm workspace.** `website/docs` has its own `package.json`, lockfile, and `wrangler.toml`; it does not join `website/web`'s workspace.
- *Why:* the docs update far more often than the marketing site; decoupling lets each pin its own Astro/Starlight versions and deploy on its own cadence. Any pnpm overrides for the docs app live in its own `pnpm-workspace.yaml` (project rule: overrides must live there, not in `package.json`).
- *Alternative:* one workspace shares deps and tooling but forces a single Astro version and reintroduces coupling. Rejected.

**D3. Match the marketing visual identity.** The docs adopt `www`'s color tokens, fonts, and logo via Starlight theming (custom CSS, `components` overrides for header/footer where needed), restyling `starlight-theme-obsidian` to those tokens rather than shipping it stock.
- *Why:* one product, two origins; consistency reduces the "I left the site" feeling when a visitor crosses from `www` to `docs`.
- *Alternative:* stock Starlight + Obsidian theme is less effort but visually disjoint. Rejected.

**D4. SEO via a Search Console Domain property + 301 redirects.** Register a DNS-verified Domain property for `deepcausality.com` (covers `www`, `docs`, apex). The docs origin serves its own `sitemap-index.xml` (via `@astrojs/sitemap`, `site: https://docs.deepcausality.com`) and `robots.txt`; docs pages canonicalize to the `docs.` origin; `www/docs/*` 301-redirects to `docs.deepcausality.com/*`; `www` and `docs` cross-link.
- *Why:* Google treats a subdomain as a separate site, so the new origin needs its own sitemap and registration, and the existing indexed `/docs/*` must be redirected to preserve earned ranking and avoid duplicate content.
- *Alternative:* per-origin URL-prefix properties (more properties to manage) or relying on auto-discovery (unreliable, loses ranking). Rejected.

**D5. Local PDF, committed.** `starlight-to-pdf` runs from a local script before push; the generated PDF is committed and served as a static asset. The Cloudflare build never launches a browser.
- *Why:* the plugin drives headless Puppeteer, and Cloudflare's build environment has no browser. Committing keeps docs and PDF consistent in the repo.
- *Alternative:* a CI step with a browser image; impractical given both sites build on Cloudflare. Rejected; revisit only if a non-Cloudflare CI runner is adopted.

**D6. Examples stay on `www`.** `examples/en/*` and the `/examples/*` routes remain on the marketing site with their current components and dropdown.
- *Why:* they are tightly coupled to marketing-site components; moving them now widens the migration. Docs↔examples references become cross-origin absolute URLs.
- *Alternative:* move examples into Starlight; deferred to a later change.

## Risks / Trade-offs

- [Astro 6 vs Starlight / plugin compatibility] → Verify the current Starlight, `starlight-theme-obsidian`, and `starlight-to-pdf` releases support the Astro version chosen for `website/docs` **before** scaffolding. The standalone app can pin whatever Astro version Starlight requires, independent of `website/web`.
- [Committed PDF goes stale] → A docs edit pushed without regenerating the PDF ships an inconsistent artifact. Mitigate with a documented pre-push script and, optionally, a checked-in marker (content hash) that a lightweight check compares against, flagging a stale PDF.
- [Lost ranking / duplicate content during migration] → 301 redirects from every old `www/docs/*` path, canonicals to the `docs.` origin, and prompt sitemap submission. Drop `/docs/*` from the `www` sitemap in the same release that adds the redirects.
- [Broken internal links] → Migrated pages move from `/docs/concepts/x/` style to the docs origin's root-relative tree; docs↔examples links become absolute cross-origin URLs. A find/replace pass plus a link check on both builds before launch.
- [Authority split across two origins] → Strong two-way cross-linking (nav, footer, the new Documentation landing page) plus the Domain property reporting both origins together.
- [Search scope] → Starlight's bundled Pagefind indexes only the docs origin; site-wide search across `www` + `docs` is not automatic. Accepted: each origin searches itself.
- [Mermaid parity] → `www` uses `astro-mermaid`; confirm Starlight renders any migrated diagrams equivalently before removing them from `www`.

## Migration Plan

1. Verify Astro/Starlight/plugin version compatibility; scaffold `website/docs` (standalone), wire `site`, `@astrojs/sitemap`, and `wrangler.toml` (`deepcausality-docs`).
2. Add Expressive Code config (dual light/dark) and `starlight-theme-obsidian`; apply the marketing identity (tokens, fonts, logo); confirm highlighting and the graph view render.
3. Migrate `concepts` first (largest, most self-contained): map frontmatter, rewrite internal links, fix docs→examples links to absolute `www` URLs.
4. Migrate the deep getting-started and overview pages.
5. Add the local `starlight-to-pdf` script; verify the single PDF; commit it.
6. On `www`: add the short getting-started + short overview summaries and the Documentation landing page (links to `docs.deepcausality.com` and docs.rs); remove the migrated long-form pages and the `src/components/docs` / `src/pages/docs` rendering; drop `/docs/*` from the `www` sitemap.
7. Add 301 redirects `www/docs/* → docs.deepcausality.com/*`.
8. Cloudflare: create the `deepcausality-docs` Worker, bind `docs.deepcausality.com`, split CI by path (`website/docs/**` vs `website/web/**`).
9. Search Console: confirm/add the `deepcausality.com` Domain property; submit the docs sitemap.
10. Post-launch: watch Search Console coverage for both origins; verify redirects resolve and canonicals are correct.

**Rollback:** the migration is additive until step 6. If problems surface after launch, repoint the `docs` DNS/route off the new Worker and restore the `www/docs/*` pages and sitemap entries from version control; the redirects (step 7) are the only `www`-side change that must be reverted to fully restore the prior state.

## Open Questions

- Exact Starlight sidebar grouping for the migrated tree (autogenerated from directories vs an explicit `sidebar` config). Resolve during step 3.
- Whether the Documentation landing page on `www` also surfaces the PDF download link directly, or only links into the docs site. Minor; decide during step 6.
