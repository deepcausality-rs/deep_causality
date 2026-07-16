#!/usr/bin/env bash
# Regenerate docs/lean.md and docs/lake.md from stardoc output. Run after
# changing rule docstrings. Invoked via `bazel run //docs:update`.
set -euo pipefail

if [[ -z "${BUILD_WORKSPACE_DIRECTORY:-}" ]]; then
  echo "error: must be invoked via 'bazel run //docs:update'" >&2
  exit 1
fi

RUNFILES_DIR="${RUNFILES_DIR:-$0.runfiles}"
LEAN_GEN="$(find "$RUNFILES_DIR" -name lean.md.generated -print -quit)"
LAKE_GEN="$(find "$RUNFILES_DIR" -name lake.md.generated -print -quit)"

cp "$LEAN_GEN" "$BUILD_WORKSPACE_DIRECTORY/docs/lean.md"
cp "$LAKE_GEN" "$BUILD_WORKSPACE_DIRECTORY/docs/lake.md"

echo "docs/lean.md and docs/lake.md regenerated."
