#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}"

BRANCH_NAME="${GITHUB_REF_NAME:-$(git rev-parse --abbrev-ref HEAD)}"
if [[ ! "${BRANCH_NAME}" =~ ^prebuilts-extras- ]]; then
  echo "Branch name must start with 'prebuilts-extras-' (got ${BRANCH_NAME})" >&2
  exit 1
fi

BRANCH_PAYLOAD="${BRANCH_NAME#prebuilts-extras-}"

bazel \
  --bazelrc=".github/workflows/ci.bazelrc" \
  build \
  --remote_header=x-buildbuddy-api-key=4jtaxdhxtyu4ylxdEwI7 \
  --@libarchive//:use_mbedtls=true \
  --config=remote \
  --config=release \
  --remote_download_outputs=toplevel \
  //prebuilt/extras:for_all_platforms

rm -rf release
mkdir -p release

PLATFORMS=(
  linux_amd64_musl linux-amd64-musl
  linux_arm64_musl linux-arm64-musl
  macos_arm64      darwin-arm64
  macos_amd64      darwin-amd64
  windows_amd64    windows-amd64-gnu
  windows_arm64    windows-arm64-gnu
)

for ((i=0; i<${#PLATFORMS[@]}; i+=2)); do
  platform_dir=${PLATFORMS[i]}
  archive_suffix=${PLATFORMS[i+1]}

  src="bazel-out/${platform_dir}-opt/bin/prebuilt/extras/extra_bins.tar.zst"
  dest="release/toolchain-extra-prebuilts-${BRANCH_PAYLOAD}-${archive_suffix}.tar.zst"

  if [[ ! -f "${src}" ]]; then
    echo "Expected archive not found at ${src}" >&2
    exit 1
  fi

  cp "${src}" "${dest}"
done

(cd release && shasum -a 256 *.tar.zst > SHA256.txt)

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  echo "version=${BRANCH_PAYLOAD}" >> "${GITHUB_OUTPUT}"
fi
