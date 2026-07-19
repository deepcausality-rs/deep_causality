# DeepCausality documentation (Starlight)

Standalone Starlight site served at **https://docs.deepcausality.com** by its own
Cloudflare Worker (`deepcausality-docs`), independent of the main website in
`../web`. See `../../openspec/changes/migrate-docs-to-starlight/` for the full plan.

## Commands

| Command          | Action                                                            |
| ---------------- | ----------------------------------------------------------------- |
| `pnpm install`   | Install dependencies                                              |
| `pnpm dev`       | Dev server at `localhost:4321` (live reload)                      |
| `pnpm build`     | Static build to `dist/`                                           |
| `pnpm preview`   | Serve the built `dist/` locally                                   |
| `pnpm pdf`       | Render the whole site to `public/deepcausality-docs.pdf` (local)  |

The site is also built under Bazel: `bazel build //website/docs:build` runs the
same `astro build` hermetically and writes `dist/` to `bazel-bin/website/docs/`.

## Single PDF (local only)

`pnpm pdf` runs `scripts/build-pdf.sh`, which builds, previews, and renders the
docs to one PDF via `npx starlight-to-pdf` (headless Chromium). It is intentionally
**not** part of the Cloudflare build (no browser there). Regenerate and commit
`public/deepcausality-docs.pdf` before pushing so the docs and PDF stay consistent.


## Fonts

Vendored locally in `public/fonts/` (Geist, JetBrains Mono) with `@font-face` in
`src/styles/fonts.css` — no CDN, no render-blocking external requests. Identity
tokens are mapped onto Starlight variables in `src/styles/theme.css`.
