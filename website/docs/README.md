# DeepCausality documentation (Starlight)

Standalone Starlight site served at **https://docs.deepcausality.com** by its own
Cloudflare Worker (`deepcausality-docs`), independent of the main website in
`../web`. See `../../openspec/changes/archive/migrate-docs-to-starlight/` for the full plan.

## Commands

| Command          | Action                                                            |
| ---------------- | ----------------------------------------------------------------- |
| `pnpm install`   | Install dependencies                                              |
| `pnpm dev`       | Dev server at `localhost:4321` (live reload)                      |
| `pnpm build`     | Static build to `dist/`                                           |
| `pnpm preview`   | Serve the built `dist/` locally                                   |
| `pnpm pdf`       | Build, serve, and crawl the working tree into a single PDF         |

The site is also built under Bazel: `bazel build //website/docs:build` runs the
same `astro build` hermetically and writes `dist/` to `bazel-bin/website/docs/`.

## Single PDF (local only)

`pnpm pdf` runs `scripts/build-pdf.sh`, which renders the whole documentation
site to one PDF via `npx starlight-to-pdf` (headless Chromium). It is
intentionally **not** part of the Cloudflare build, which provides no browser.

The script reads the **working tree**, not the deployed site. It runs the Astro
build, starts a local preview, crawls that, and writes
`public/deepcausality-docs.pdf`. So no deploy has to land first:

1. write the documentation changes
2. run `pnpm pdf`
3. commit the regenerated `public/deepcausality-docs.pdf`

`starlight-to-pdf` has no base-URL rewrite: the origin it crawls is the origin it
bakes into every link annotation it writes. Crawling a local preview therefore
used to ship a download whose in-body cross-references all pointed at a dead
`http://localhost:4329`. The script now drops those annotations instead of
rewriting them, keeping the link text — a cross-reference from one page of the
PDF to another page of the same PDF gains nothing from also being a web link.
External links (GitHub, papers, the main site) are absolute and correct whatever
the crawl host, so they survive untouched.

That post-processing step needs `python3` with `pypdf`:

```bash
python3 -m pip install --user pypdf
```

It writes to a temporary file and replaces the committed PDF only once the page
count matches, so a rejected result cannot leave a truncated file behind.

The preview binds `PDF_PREVIEW_PORT`, default `4321`. The script refuses to start
when anything already listens on that port, and then waits for a server that
returns the `dist/index.html` it has just built, byte for byte. Without those two
checks a stray server on the same port would be crawled into the committed PDF
instead. A leftover preview is the usual reason for the refusal; stop it with
`pnpm exec astro preview stop`, or point `PDF_PREVIEW_PORT` at a free port.

Puppeteer downloads its own Chromium on first run. Set `CHROME_PATH` to use an
installed browser instead, which is the fix when that download is unusable:

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" pnpm pdf
```

A full crawl takes about 20 seconds and yields roughly 173 pages. Runs
occasionally abort on a navigation timeout; a rerun is enough.


## Fonts

Vendored locally in `public/fonts/` (Geist, JetBrains Mono) with `@font-face` in
`src/styles/fonts.css` — no CDN, no render-blocking external requests. Identity
tokens are mapped onto Starlight variables in `src/styles/theme.css`.
