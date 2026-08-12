#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
set -o errexit
set -o nounset
set -o pipefail

# Re-vendor the Bazel module sources checked into thirdparty/bzlmod/.
#
# Run this after bumping any of the bazel_dep versions in MODULE.bazel that
# have a matching local_path_override, otherwise the override keeps pinning the
# old vendored tree and the version bump silently has no effect.
#
# Sources are copied from the RESOLVED external repos rather than from upstream
# release tarballs, so the BCR remote_patches (aspect_bazel_lib,
# toolchains_buildbuddy and llvm all carry one) are already applied. Copying
# from GitHub instead would silently drop them.
#
# Only Starlark rule sources are vendored -- 5.4 MB total. The toolchains these
# rules configure (Rust ~1.8 GB, LLVM ~1 GB, Lean/Mathlib ~10 GB) come from
# module extensions, which local_path_override cannot vendor; those belong in
# the CI repository cache.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
THIRDPARTY="${REPO_ROOT}/thirdparty/bzlmod"

# Modules to vendor. rules_rust is deliberately absent: it is pinned to a fork
# via git_override while https://github.com/bazelbuild/rules_rust/pull/4220 is
# open, and the two override kinds are mutually exclusive.
MODULES=(
  aspect_bazel_lib
  bazel_skylib
  platforms
  toolchains_buildbuddy
  llvm
)

cd "${REPO_ROOT}"

# The overrides have to be inactive while re-vendoring, otherwise Bazel resolves
# the module to the tree we are about to overwrite instead of to the registry.
if grep -q 'path = "thirdparty/bzlmod/llvm"' MODULE.bazel; then
  echo "ERROR: the local_path_override entries in MODULE.bazel are active." >&2
  echo "       Comment out the vendored-module override block, run this script," >&2
  echo "       then restore it." >&2
  exit 1
fi

EXTERNAL="$(bazel info output_base)/external"

for module in "${MODULES[@]}"; do
  # Most modules materialize as '<name>+'; a few well-known ones (platforms)
  # keep their bare name.
  if [[ -d "${EXTERNAL}/${module}+" ]]; then
    src="${EXTERNAL}/${module}+"
  elif [[ -d "${EXTERNAL}/${module}" ]]; then
    src="${EXTERNAL}/${module}"
  else
    echo "ERROR: ${module} is not materialized under ${EXTERNAL}." >&2
    echo "       Run: bazel fetch --repo=@@${module}+" >&2
    exit 1
  fi

  rm -rf "${THIRDPARTY:?}/${module}"
  cp -RL "${src}" "${THIRDPARTY}/${module}"
  find "${THIRDPARTY}/${module}" -name "MODULE.bazel.lock" -delete
  rm -rf "${THIRDPARTY}/${module}/.git"

  printf '%-24s %s\n' "${module}" "$(du -shL "${THIRDPARTY}/${module}" | cut -f1)"
done

echo
echo "Re-vendored ${#MODULES[@]} modules. Restore the override block in MODULE.bazel,"
echo "then verify with: bazel build --nobuild //..."
