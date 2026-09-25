#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# Runs the release pre-flight: both checks that cover the dependency graph.
#
# check_dev_deps.sh asserts that crates.io holds every internal dev-dependency version the
# workspace requires. Its header explains why the dry run below cannot see that.
#
# `cargo publish --workspace --dry-run` packages and builds every crate from its own `.crate`, so
# it catches a missing file, a broken `exclude`, and a crate that fails to compile once unpacked.
#
# A green run says nothing about the registry token, a malformed changelog, or a crates.io outage
# mid-release.
#
# Run this on a clean tree. Cargo refuses to package a crate holding uncommitted changes, which is
# what CI always sees and what a release publishes.
set -o errexit
set -o nounset
set -o pipefail

source "$(dirname "${BASH_SOURCE[0]}")/crates.sh"
cd "$DC_REPO_ROOT"

echo "== internal dev-dependencies must be on crates.io =="
bash scripts/check_dev_deps.sh

echo
echo "== cargo publish --workspace --dry-run =="
# `--locked` fails the check on a stale Cargo.lock rather than updating it, so the run reflects the
# versions a release would publish.
cargo publish --workspace --dry-run --locked

echo
echo "publish pre-flight passed for ${#DC_CRATES[@]} crates"
