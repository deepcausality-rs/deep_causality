#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}"

LLVM_VERSION=$(python3 - <<'PY'
import re
from pathlib import Path

match = re.search(r'^LLVM_VERSION\s*=\s*"([^"]+)"', Path("MODULE.bazel").read_text(), re.M)
if not match:
    raise SystemExit("Could not find LLVM_VERSION in MODULE.bazel")
print(match.group(1))
PY
)

BRANCH_NAME="${GITHUB_REF_NAME:-$(git rev-parse --abbrev-ref HEAD)}"
if [[ ! "${BRANCH_NAME}" =~ ^llvm- ]]; then
  echo "Branch name must start with 'llvm-' (got ${BRANCH_NAME})" >&2
  exit 1
fi

BRANCH_PAYLOAD="${BRANCH_NAME#llvm-}"
BASE_BRANCH_VERSION="${BRANCH_PAYLOAD%-*}"

if [[ "${BASE_BRANCH_VERSION}" != "${LLVM_VERSION}" ]]; then
  echo "Branch version '${BASE_BRANCH_VERSION}' does not match LLVM_VERSION '${LLVM_VERSION}'" >&2
  exit 1
fi

bazel \
  --bazelrc=".github/workflows/ci.bazelrc" \
  build \
  --remote_header=x-buildbuddy-api-key=4jtaxdhxtyu4ylxdEwI7 \
  --config=remote \
  --config=release \
  --remote_download_outputs=toplevel \
  //prebuilt/llvm:all

rm -rf release
mkdir -p release

PLATFORMS=(
  linux_amd64_musl linux-amd64-musl llvm_release
  linux_arm64_musl linux-arm64-musl llvm_release
  macos_arm64      darwin-arm64     llvm_release
  macos_amd64      darwin-amd64     llvm_release
  windows_amd64    windows-amd64    windows_llvm_release
  windows_arm64    windows-arm64    windows_llvm_release
)

for ((i=0; i<${#PLATFORMS[@]}; i+=3)); do
  platform_dir=${PLATFORMS[i]}
  archive_suffix=${PLATFORMS[i+1]}
  target_name=${PLATFORMS[i+2]}

  src="bazel-out/${platform_dir}-opt/bin/prebuilt/llvm/${target_name}.tar.zst"
  dest="release/llvm-toolchain-minimal-${LLVM_VERSION}-${archive_suffix}.tar.zst"

  if [[ ! -f "${src}" ]]; then
    echo "Expected archive not found at ${src}" >&2
    exit 1
  fi

  cp "${src}" "${dest}"
done

(cd release && shasum -a 256 *.tar.zst > SHA256.txt)

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  echo "version=${LLVM_VERSION}" >> "${GITHUB_OUTPUT}"
fi
