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
#     when anything already listens there, and then waits for a server that returns
#     the dist/index.html it just built, so a stray server cannot end up in the PDF.
#   - A leftover preview is the usual reason for that refusal. Stop it with
#     `pnpm exec astro preview stop`, or point PDF_PREVIEW_PORT elsewhere.
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

# `lsof` names the process holding the port, which is what the ownership check
# below needs. It ships with macOS. Where it is missing, the port checks fall back
# to probes that answer only "occupied or not".
HAVE_LSOF=
if command -v lsof >/dev/null 2>&1; then
  HAVE_LSOF=1
fi

# PIDs listening on $PORT, one per line, empty when the port is free.
listener_pids() {
  lsof -nP -iTCP:"$PORT" -sTCP:LISTEN -t 2>/dev/null || true
}

# Occupancy, not HTTP success. A server that owns the port and answers / with a
# 404 or a 500 still owns the port, so the probes must not read an error status as
# a free port: `curl -f` does exactly that, which is why it is absent here.
port_is_taken() {
  if [[ -n "$HAVE_LSOF" ]]; then
    [[ -n "$(listener_pids)" ]]
  elif command -v nc >/dev/null 2>&1; then
    nc -z 127.0.0.1 "$PORT" >/dev/null 2>&1
  else
    curl -so /dev/null --max-time 2 "$BASE_URL/"
  fi
}

# Is $1 the preview we started, or something it spawned? `pnpm preview` hands off
# through a chain of processes, so the one holding the port is usually a grandchild
# of $PREVIEW_PID rather than $PREVIEW_PID itself. Walk the parent links back up.
is_our_process() {
  local pid="$1"
  local hops=0
  while [[ -n "$pid" && "$pid" != "0" && "$pid" != "1" && "$hops" -lt 10 ]]; do
    if [[ "$pid" == "$PREVIEW_PID" ]]; then
      return 0
    fi
    pid="$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d ' ' || true)"
    hops=$((hops + 1))
  done
  return 1
}

echo "==> Building the site ..."
pnpm build

# The build we just made has to identify itself, because the readiness probe below
# would otherwise accept whatever answers on $PORT and crawl it into the committed
# PDF. dist/index.html is that identity: `pnpm build` wrote it from the working
# tree seconds ago, and the preview serves the file back byte for byte.
INDEX_HTML="$(cat "$DOCS_DIR/dist/index.html" 2>/dev/null || true)"
[[ -n "$INDEX_HTML" ]] || {
  echo "error: dist/index.html is missing or empty; cannot identify the preview server" >&2
  exit 1
}

# Refuse to share the port rather than guess. An unrelated server already bound
# here would leave the preview nowhere to bind, and killing it is not ours to do.
if port_is_taken; then
  echo "error: something already holds port $PORT." >&2
  echo "       Stop it, or point PDF_PREVIEW_PORT at a free port." >&2
  exit 1
fi

echo "==> Starting a local preview on $BASE_URL ..."
pnpm preview --port "$PORT" >/tmp/starlight-pdf-preview.log 2>&1 &
PREVIEW_PID=$!

# Always take the preview server down, whether the crawl succeeds or fails. Killing
# $PREVIEW_PID alone does not do it. The process listening is a child, which outlives
# its parent, and `astro preview` may have detached it altogether; either way the port
# stays bound and the next run refuses to start. So take down what holds the port,
# and do it before killing the parent, while the parent links still lead back here.
#
# Two ways to recognise what to take down. The parent links cover the foreground
# case. The command line covers the detached one: the port was free when this run
# checked it, so an astro preview holding it now is the one this run started.
# Anything else on the port is left alone.
cleanup() {
  if [[ -n "$HAVE_LSOF" ]]; then
    local pid
    for pid in $(listener_pids); do
      if is_our_process "$pid" || [[ "$(ps -o command= -p "$pid" 2>/dev/null || true)" == *astro*preview* ]]; then
        kill "$pid" 2>/dev/null || true
      fi
    done
  fi
  kill "$PREVIEW_PID" 2>/dev/null || true
}
trap cleanup EXIT

# Wait for a server that returns this build, byte for byte, rather than for any
# server that returns a page. The old test looked for the site's <title>, which is
# the same string in every build of the site and so accepted any preview of it,
# including one of a different checkout. Comparing the whole of dist/index.html
# pins the answer to the tree we just built. The response is captured rather than
# piped into grep, because `grep -q` exits early and `set -o pipefail` would then
# read curl's SIGPIPE as a failed probe.
#
# Where the preview runs in the foreground, lsof and the parent links prove more
# than the bytes do, so the loop records that proof when it is available. It often
# is not: `astro preview` detaches into a background daemon whenever it detects an
# agent driving the terminal, or when --background is passed, and a detached server
# keeps no parent link back to this run.
#
# What the check does not cover. A second preview of this same dist/ directory
# serves identical bytes and is accepted. That costs nothing: the pages it renders
# are the pages ours would have rendered. The real gap is timing. The question is
# settled before the crawl rather than during it, so a server that takes the port
# after this loop passes is a server the crawl reads. No shell script closes that
# window.
READY=
OWNED=
for _ in $(seq 1 60); do
  body="$(curl -sf --max-time 5 "$BASE_URL/" || true)"
  if [[ "$body" == "$INDEX_HTML" ]]; then
    READY=1
    if [[ -n "$HAVE_LSOF" ]]; then
      for pid in $(listener_pids); do
        if is_our_process "$pid"; then
          OWNED=1
        fi
      done
    fi
    break
  fi
  sleep 1
done
[[ -n "$READY" ]] || {
  echo "error: nothing on $BASE_URL served the site just built into dist/, after 60s." >&2
  if [[ -n "$HAVE_LSOF" ]]; then
    HOLDERS="$(listener_pids | tr '\n' ' ')"
    if [[ -n "$HOLDERS" ]]; then
      echo "       Port $PORT is held by PID(s) $HOLDERS, serving something else." >&2
    fi
  fi
  echo "       See /tmp/starlight-pdf-preview.log" >&2
  exit 1
}
if [[ -n "$OWNED" ]]; then
  echo "    Serving the preview this run started (PID on $PORT descends from $PREVIEW_PID)."
else
  echo "    Serving this build, matched by content: nothing ties the process on $PORT to this run."
fi

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
    # This also runs after a successful os.replace, where the temp file is already
    # gone. `unlink(missing_ok=True)` would say that in one line and would need
    # Python 3.8; on 3.7 it raises TypeError, which from a `finally` turns a run
    # that worked into a failure. Catching the error costs nothing and holds the
    # floor at 3.6, which is what the rest of this script needs.
    try:
        tmp_path.unlink()
    except FileNotFoundError:
        pass

print(f"    Stripped {stripped} internal link(s); kept {kept} external link(s).")
if stripped == 0:
    print(f"    Warning: nothing matched {base_url} - check the crawl host.")
PYTHON

echo "==> Done: $OUT_DIR/deepcausality-docs.pdf"
echo "    Commit the regenerated PDF so docs and PDF stay consistent."
