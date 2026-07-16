#!/usr/bin/env bash
# Assert that a lean_olean_archive tarball contains the expected compiled
# olean at its import-root-relative path. Invoked as a sh_test.
set -euo pipefail

TARBALL="$1"
WANT="Lib/Thing.olean"

listing="$(tar tzf "$TARBALL")"
# Entries are prefixed with "./" by `tar -C <root> .`.
if printf '%s\n' "$listing" | grep -qE "(^|/)${WANT}\$"; then
  echo "OK: archive contains ${WANT}"
else
  echo "FAIL: ${WANT} not in archive. Contents:" >&2
  printf '%s\n' "$listing" >&2
  exit 1
fi
