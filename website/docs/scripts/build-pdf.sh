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
# Workflow:
#   1. run `npm run pdf` locally
#   2. commit the regenerated public/deepcausality-docs.pdf
#   3. push (Cloudflare serves the committed PDF; it launches no browser)
#
# Output: public/deepcausality-docs.pdf
#
# Notes:
#   - The table of contents uses `--contents-links internal`, so TOC entries
#     navigate to headings within the PDF (not to the crawl origin).
#   - In-body cross-references resolve to the crawl origin (the local preview).
#     For the published PDF, regenerate against the live site once it is up so
#     those links point to https://docs.deepcausality.com rather than localhost.
set -euo pipefail

PORT=4329
DOCS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$DOCS_DIR/public"

echo "==> Building docs site..."
npx astro build

echo "==> Starting preview server on :$PORT ..."
npx astro preview --port "$PORT" >/dev/null 2>&1 &
PREVIEW_PID=$!
trap 'kill "$PREVIEW_PID" 2>/dev/null || true' EXIT

echo "==> Waiting for preview server ..."
for _ in $(seq 1 30); do
  if curl -sf "http://localhost:$PORT/" >/dev/null 2>&1; then break; fi
  sleep 1
done

echo "==> Rendering single PDF (downloads Chromium via npx on first run) ..."
npx --yes starlight-to-pdf "http://localhost:$PORT" \
  --filename deepcausality-docs \
  --path "$OUT_DIR" \
  --contents-links internal \
  --print-bg

echo "==> Done: $OUT_DIR/deepcausality-docs.pdf"
echo "    Commit the regenerated PDF so docs and PDF stay consistent."
