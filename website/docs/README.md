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
| `pnpm pdf`       | Render the deployed site to `public/deepcausality-docs.pdf`       |

The site is also built under Bazel: `bazel build //website/docs:build` runs the
same `astro build` hermetically and writes `dist/` to `bazel-bin/website/docs/`.

## Single PDF (local only)

`pnpm pdf` runs `scripts/build-pdf.sh`, which renders the whole documentation
site to one PDF via `npx starlight-to-pdf` (headless Chromium). It is
intentionally **not** part of the Cloudflare build, which provides no browser.

The script crawls the deployed site at **https://docs.deepcausality.com**. It
builds nothing locally and starts no preview server. `starlight-to-pdf` has no
base-URL rewrite, so the origin it crawls is the origin it writes into every
link annotation of the PDF. Crawling a local preview shipped a download whose
in-body cross-references pointed at `http://localhost:4329`, dead for every
reader who opened it.

The PDF therefore shows what is deployed rather than what sits in the working
tree. Order the steps accordingly:

1. deploy the documentation changes
2. run `pnpm pdf`
3. commit the regenerated `public/deepcausality-docs.pdf`

Regenerating before the deploy lands reproduces the previous site.

Puppeteer downloads its own Chromium on first run. Set `CHROME_PATH` to use an
installed browser instead, which is the fix when that download is unusable:

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" pnpm pdf
```

A full crawl takes about 20 seconds and yields roughly 165 pages. Runs
occasionally abort on a navigation timeout; the script writes the PDF only on
success, so the committed file survives a failed run and a rerun is enough.


## Fonts

Vendored locally in `public/fonts/` (Geist, JetBrains Mono) with `@font-face` in
`src/styles/fonts.css` — no CDN, no render-blocking external requests. Identity
tokens are mapped onto Starlight variables in `src/styles/theme.css`.
