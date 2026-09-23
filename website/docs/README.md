# DeepCausality documentation (Starlight)

Standalone Starlight site served at **https://docs.deepcausality.com** by its own
Cloudflare Worker (`deepcausality-docs`), independent of the project website in
`../web`. See `../../openspec/changes/archive/migrate-docs-to-starlight/` for the full plan.

## Commands

| Command          | Action                                                            |
| ---------------- | ----------------------------------------------------------------- |
| `pnpm install`   | Install dependencies                                              |
| `pnpm dev`       | Dev server at `localhost:4321` (live reload)                      |
| `pnpm build`     | Static build to `dist/`                                           |
| `pnpm preview`   | Serve the built `dist/` locally                                   |
| `pnpm pdf`       | Build, serve, and crawl the working tree into a single PDF         |

## Architecture map

`public/architecture.html` is a standalone interactive diagram of the
29-crate workspace, served at `/architecture.html` and linked from
[Overview -> Architecture](src/content/docs/overview/architecture.md). It is a
committed generated artifact, like the PDF.

The source is `scripts/workspace-architecture.json`: a typed specification of
nodes, boundaries, relationships, and cards, rendered by the `archify` tool.
Every node names the repository files it stands for. The renderer checks each
path against the repository before writing the page, so a node whose source is
gone fails the render.

To update the map, edit the specification, re-render it over
`public/architecture.html`, and commit both files. The page carries its own
header, theme switch, and legend; the docs site sets
`X-Frame-Options: DENY`, so link to it rather than embedding it in an iframe.

## Single PDF (local only)

`pnpm pdf` runs `scripts/build-pdf.sh`, which renders the whole documentation
site to one PDF via `npx starlight-to-pdf` (headless Chromium). It is **not**
part of the Cloudflare build, which provides no browser.

The script reads the **working tree**, not the deployed site. It runs the Astro
build, starts a local preview, crawls that, and writes
`public/deepcausality-docs.pdf`, so no deploy has to land first:

1. write the documentation changes
2. run `pnpm pdf`
3. commit the regenerated `public/deepcausality-docs.pdf`

`starlight-to-pdf` has no base-URL rewrite: the origin it crawls is the origin it
bakes into every link annotation it writes, so a crawl of a local preview would
point every in-body cross-reference at a dead localhost address. The script drops
those annotations and keeps the link text. External links (GitHub, papers, the
project website) are absolute and correct whatever the crawl host, so they
survive untouched.

That post-processing step needs `python3` with `pypdf`:

```bash
python3 -m pip install --user pypdf
```

It writes to a temporary file and replaces the committed PDF only once the page
count matches, so a rejected result cannot leave a truncated file behind.

The preview binds `PDF_PREVIEW_PORT`, default `4321`. The script refuses to start
when anything already listens on that port, and then waits for a server that
returns the `dist/index.html` it has just built, byte for byte, so a stray server
on the same port never ends up in the committed PDF. A leftover preview is the usual reason for the refusal; stop it with
`pnpm exec astro preview stop`, or point `PDF_PREVIEW_PORT` at a free port.

Puppeteer downloads its own Chromium on first run. If that download is unusable,
set `CHROME_PATH` to an installed browser:

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" pnpm pdf
```

A full crawl takes about 20 seconds and yields roughly 173 pages. Runs
occasionally abort on a navigation timeout; a rerun is enough.


## Fonts

Vendored locally in `public/fonts/` (Geist, JetBrains Mono) with `@font-face` in
`src/styles/fonts.css`: no CDN, no render-blocking external requests.
`src/styles/theme.css` maps the identity tokens onto Starlight variables.
