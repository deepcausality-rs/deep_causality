#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
set -o errexit
set -o nounset
set -o pipefail

# Bazel file formatting (Installed via homebrew)
# https://github.com/bazelbuild/buildtools
buildifier -r MODULE.bazel BUILD.bazel thirdparty/BUILD.bazel
source "$(dirname "${BASH_SOURCE[0]}")/crates.sh"
for CRATE_DIR in "${DC_CRATE_DIRS[@]}"; do
    buildifier -r "$CRATE_DIR"/
done

# Code formatting
# https://github.com/rust-lang/rustfmt
command cargo fmt --all