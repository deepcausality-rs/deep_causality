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
buildifier -r deep_causality/
buildifier -r deep_causality_algorithms/
buildifier -r deep_causality_data_structures/
buildifier -r deep_causality_discovery/
buildifier -r deep_causality_haft/
buildifier -r deep_causality_macros/
buildifier -r deep_causality_multivector/
buildifier -r deep_causality_num/
buildifier -r deep_causality_rand/
buildifier -r deep_causality_tensor/
buildifier -r deep_causality_uncertain/
buildifier -r examples/
buildifier -r ultragraph/

# Code formatting
# https://github.com/rust-lang/rustfmt
command cargo fmt --all