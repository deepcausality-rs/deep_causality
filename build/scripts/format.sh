#
# SPDX-License-Identifier: MIT
# Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
#
set -o errexit
set -o nounset
set -o pipefail

# Bazel file formatting (Installed via homebrew)
# https://github.com/bazelbuild/buildtools
buildifier -r MODULE.bazel BUILD.bazel thirdparty/BUILD.bazel
buildifier -r dcl_data_structures/
buildifier -r deep_causality/
buildifier -r deep_causality_macros/
buildifier -r ultragraph/


# Code formatting
# https://github.com/rust-lang/rustfmt
command cargo fmt --all