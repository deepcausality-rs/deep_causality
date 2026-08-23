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
# The crawl target is a LOCAL PREVIEW of the working tree, so the PDF reflects
# what you have written rather than what is currently deployed. Edit, run
# `pnpm pdf`, commit — no deploy required first.
#
# That used to be impossible. starlight-to-pdf has no base-URL rewrite: it bakes
# an absolute URL into every body cross-reference, using whatever host it
# crawled, and its only link-mode flag (`--contents-links`) governs the table of
# contents rather than body links. Crawling localhost therefore produced a PDF
# full of dead `http://localhost:<port>/...` links, which is why this script
# used to read the deployed site instead.
#
# The fix is to drop those links rather than rewrite them. A cross-reference
# from one page of the PDF to another page of the same PDF earns nothing by
# also being a web link, so the inline post-processing step below removes the
# link action and leaves the text. External links (GitHub, papers, the main site) are
# absolute and correct regardless of crawl host, so they are kept. The table of
# contents is untouched: `--contents-links internal` makes it internal PDF
# destinations rather than URI actions.
#
# Workflow:
#   1. run `pnpm pdf` locally
#   2. commit the regenerated public/deepcausality-docs.pdf
#
# Output: public/deepcausality-docs.pdf
#
# Requirements:
#   - python3 with pypdf   (python3 -m pip install --user pypdf), used inline below
#     to strip the internal links from the rendered PDF.
#
# Notes:
#   - The preview binds PDF_PREVIEW_PORT (default 4321). The script refuses to run
#     when that port is already taken, and checks that the server answering on it
#     is the one it started, so a stray server cannot end up in the PDF.
#   - Set CHROME_PATH to use an installed browser instead of the one Puppeteer
#     downloads on first run (e.g. when that download is unusable):
#       CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" pnpm pdf
set -euo pipefail

PORT="${PDF_PREVIEW_PORT:-4321}"
BASE_URL="http://localhost:$PORT"
DOCS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$DOCS_DIR/public"

cd "$DOCS_DIR"

command -v python3 >/dev/null || { echo "error: python3 not found" >&2; exit 1; }
python3 -c 'import pypdf' 2>/dev/null || {
  echo "error: pypdf not installed. Run: python3 -m pip install --user pypdf" >&2
  exit 1
}

echo "==> Building the site ..."
pnpm build

# The site we just built has to identify itself, because the readiness probe below
# would otherwise accept whatever answers on $PORT and crawl it into the committed
# PDF. The <title> of the page in dist/ is the marker.
SITE_MARKER="$(sed -n 's/.*<title>\([^<]*\)<\/title>.*/\1/p' "$DOCS_DIR/dist/index.html" | head -n 1)"
[[ -n "$SITE_MARKER" ]] || {
  echo "error: no <title> in dist/index.html; cannot identify the preview server" >&2
  exit 1
}

# Refuse to share the port rather than guess. An unrelated server already bound
# here would make the two checks below ambiguous, and killing it is not ours to do.
if curl -sfo /dev/null --max-time 2 "$BASE_URL/"; then
  echo "error: something is already serving $BASE_URL." >&2
  echo "       Stop it, or point PDF_PREVIEW_PORT at a free port." >&2
  exit 1
fi

echo "==> Starting a local preview on $BASE_URL ..."
pnpm preview --port "$PORT" >/tmp/starlight-pdf-preview.log 2>&1 &
PREVIEW_PID=$!
# Always take the preview server down, whether the crawl succeeds or fails.
trap 'kill "$PREVIEW_PID" 2>/dev/null || true' EXIT

# Wait for *our* preview, not merely for a responder: the body has to carry the
# marker taken from dist/. The response is captured rather than piped into grep,
# because `grep -q` exits early and `set -o pipefail` would then read curl's
# SIGPIPE as a failed probe.
READY=
for _ in $(seq 1 60); do
  body="$(curl -sf --max-time 5 "$BASE_URL/" || true)"
  if [[ "$body" == *"<title>$SITE_MARKER</title>"* ]]; then
    READY=1
    break
  fi
  sleep 1
done
[[ -n "$READY" ]] || {
  echo "error: nothing serving \"$SITE_MARKER\" on $BASE_URL after 60s." >&2
  echo "       See /tmp/starlight-pdf-preview.log" >&2
  exit 1
}

# Note the `${arr[@]+...}` guard: under `set -u`, bash 3.2 (the system bash on
# macOS) treats an empty array's expansion as an unbound variable and aborts.
BROWSER_ARGS=()
if [[ -n "${CHROME_PATH:-}" ]]; then
  BROWSER_ARGS=(--browser-executable "$CHROME_PATH")
fi

echo "==> Rendering single PDF from $BASE_URL (downloads Chromium via npx on first run) ..."
npx --yes starlight-to-pdf "$BASE_URL" \
  --filename deepcausality-docs \
  --path "$OUT_DIR" \
  --contents-links internal \
  --print-bg \
  ${BROWSER_ARGS[@]+"${BROWSER_ARGS[@]}"}

echo "==> Removing the internal cross-reference links ..."
python3 - "$OUT_DIR/deepcausality-docs.pdf" "$BASE_URL" <<'PYTHON'
import os
import sys
from pathlib import Path

try:
    from pypdf import PdfReader, PdfWriter
    from pypdf.generic import ArrayObject, NameObject
except ImportError:
    sys.exit("error: pypdf is required. Run: python3 -m pip install --user pypdf")

pdf_path, base_url = Path(sys.argv[1]), sys.argv[2].rstrip("/")
if not pdf_path.is_file():
    sys.exit(f"error: no such file: {pdf_path}")

before = len(PdfReader(str(pdf_path)).pages)
writer = PdfWriter(clone_from=str(pdf_path))

stripped = kept = 0
for page in writer.pages:
    annots = page.get("/Annots")
    if not annots:
        continue
    surviving = []
    for ref in annots:
        try:
            obj = ref.get_object()
        except Exception:
            surviving.append(ref)
            continue
        action = obj.get("/A")
        uri = str(action.get("/URI")) if action and action.get("/URI") else None
        if uri is not None and uri.startswith(base_url):
            # An internal cross-reference. Drop the action; the link text stays.
            stripped += 1
            continue
        if uri is not None:
            kept += 1
        surviving.append(ref)
    page[NameObject("/Annots")] = ArrayObject(surviving)

# Write beside the target and swap in only what validates. Writing in place would
# leave the committed PDF truncated if `write` failed midway, and would leave a
# modified file behind even when the page-count check below rejects the result.
tmp_path = pdf_path.with_name(pdf_path.name + ".tmp")
try:
    with tmp_path.open("wb") as handle:
        writer.write(handle)

    after = len(PdfReader(str(tmp_path)).pages)
    if before != after:
        sys.exit(f"error: page count changed, {before} -> {after}; refusing to continue")

    # Same directory, so this is an atomic rename rather than a copy.
    os.replace(tmp_path, pdf_path)
finally:
    # A `sys.exit` above raises SystemExit, so the half-written file goes either way.
    tmp_path.unlink(missing_ok=True)

print(f"    Stripped {stripped} internal link(s); kept {kept} external link(s).")
if stripped == 0:
    print(f"    Warning: nothing matched {base_url} - check the crawl host.")
PYTHON

echo "==> Done: $OUT_DIR/deepcausality-docs.pdf"
echo "    Commit the regenerated PDF so docs and PDF stay consistent."
