#
# SPDX-License-Identifier: MIT
# Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
#

# bin/bash
set -o errexit
set -o nounset
set -o pipefail

# Check for outdated dependencies
# Install or update with cargo install --locked cargo-outdated
# https://github.com/kbknapp/cargo-outdated
cargo outdated --workspace

# Scan for unused dependencies
cargo machete deep_causality deep_causality_algorithms deep_causality_discovery deep_causality_effects deep_causality_haft deep_causality_tensor deep_causality_rand deep_causality_physics deep_causality_sparse deep_causality_num deep_causality_data_structures deep_causality_macros deep_causality_multivector deep_causality_uncertain ultragraph

# Scan again to report all unfixed vulnerabilities
# install or update with cargo install cargo-audit --locked
# https://crates.io/crates/cargo-audit
cargo audit

# Check a package and all of its dependencies for errors.
# https://doc.rust-lang.org/cargo/FEATURES=unsafes/cargo-check.html
cargo check --all-targets --all-features

# Check for linter errors
# https://github.com/rust-lang/rust-clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check code formatting
# https://github.com/rust-lang/rustfmt
cargo fmt --all --check
