#!/usr/bin/env bash
#
# Local-only: render the entire documentation site to a single PDF.
#
# This is intentionally NOT part of the Cloudflare build. starlight-to-pdf
# drives a headless browser (Puppeteer/Chromium), which Cloudflare's build
# environment does not provide. We invoke it via `npx` so it never enters the
# project's dependencies or the deploy; the generated PDF is committed and
# served as a static asset from public/.
#
# The crawl target is the live site, not a local preview. starlight-to-pdf has
# no base-URL rewrite: every link it writes into the PDF is the URL it crawled.
# Crawling localhost therefore baked `http://localhost:<port>/...` into every
# in-body cross-reference, which is dead in a published download. Reading the
# deployed site makes those links resolve to https://docs.deepcausality.com.
#
# The consequence is that the PDF reflects what is DEPLOYED, not the working
# tree. Deploy documentation changes first, then regenerate.
#
# Workflow:
#   1. deploy the docs site
#   2. run `pnpm pdf` locally
#   3. commit the regenerated public/deepcausality-docs.pdf
#
# Output: public/deepcausality-docs.pdf
#
# Notes:
#   - The table of contents uses `--contents-links internal`, so TOC entries
#     navigate to headings within the PDF rather than back out to the site.
#   - Set CHROME_PATH to use an installed browser instead of the one Puppeteer
#     downloads on first run (e.g. when that download is unusable):
#       CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" pnpm pdf
set -euo pipefail

SITE_URL="https://docs.deepcausality.com"
DOCS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$DOCS_DIR/public"

BROWSER_ARGS=()
if [[ -n "${CHROME_PATH:-}" ]]; then
  BROWSER_ARGS=(--browser-executable "$CHROME_PATH")
fi

echo "==> Rendering single PDF from $SITE_URL (downloads Chromium via npx on first run) ..."
npx --yes starlight-to-pdf "$SITE_URL" \
  --filename deepcausality-docs \
  --path "$OUT_DIR" \
  --contents-links internal \
  --print-bg \
  "${BROWSER_ARGS[@]}"

echo "==> Done: $OUT_DIR/deepcausality-docs.pdf"
echo "    Commit the regenerated PDF so docs and PDF stay consistent."
