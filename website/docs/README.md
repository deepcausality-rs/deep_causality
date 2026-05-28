# DeepCausality documentation (Starlight)

Standalone Starlight site served at **https://docs.deepcausality.com** by its own
Cloudflare Worker (`deepcausality-docs`), independent of the marketing site in
`../web`. See `../../openspec/changes/migrate-docs-to-starlight/` for the full plan.

## Commands

| Command          | Action                                                            |
| ---------------- | ----------------------------------------------------------------- |
| `npm install`    | Install dependencies                                              |
| `npm run dev`    | Dev server at `localhost:4321` (live reload)                      |
| `npm run build`  | Static build to `dist/`                                           |
| `npm run preview`| Serve the built `dist/` locally                                   |
| `npm run pdf`    | Render the whole site to `public/deepcausality-docs.pdf` (local)  |

## Gotcha: the graph view only renders in build/preview, not dev

The backlinks **Graph View** (`starlight-site-graph`) builds its data by crawling
the **generated HTML at build time** (`dist/sitegraph/sitemap.json`). Under
`npm run dev` that crawl does not run, so the graph canvas is **empty by design**.
To see the graph, use `npm run preview` (or `npm run build`), which serves the
built site with the graph data. This is not a configuration error.

## Single PDF (local only)

`npm run pdf` runs `scripts/build-pdf.sh`, which builds, previews, and renders the
docs to one PDF via `npx starlight-to-pdf` (headless Chromium). It is intentionally
**not** part of the Cloudflare build (no browser there). Regenerate and commit
`public/deepcausality-docs.pdf` before pushing so the docs and PDF stay consistent.

## Content migration

`scripts/migrate-content.mjs` is the one-shot transform that brought the long-form
docs over from `../web` (frontmatter remap + link rewrites). It skips
`overview/why.md`, whose mermaid diagrams were hand-converted to static inline SVG.

## Fonts

Vendored locally in `public/fonts/` (Geist, JetBrains Mono) with `@font-face` in
`src/styles/fonts.css` — no CDN, no render-blocking external requests. Identity
tokens are mapped onto Starlight variables in `src/styles/theme.css`.
