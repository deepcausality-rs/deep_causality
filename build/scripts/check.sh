#
# SPDX-License-Identifier: MIT
# Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
#

# bin/bash
set -o errexit
set -o nounset
set -o pipefail

# Check for outdated dependencies
# https://github.com/kbknapp/cargo-outdated
command cargo outdated --workspace --all-features

command cargo machete deep_causality deep_causality_algorithms deep_causality_rand deep_causality_num deep_causality_data_structures deep_causality_macros deep_causality_uncertain ultragraph

# Scan for unused dependencies
# https://crates.io/crates/cargo-udeps
command cargo +nightly udeps --all-targets --all-features

# Scan again to report all unfixed vulnerabilities
# https://crates.io/crates/cargo-audit
#command cargo audit --all-targets --all-features

# Check a package and all of its dependencies for errors.
# https://doc.rust-lang.org/cargo/FEATURES=unsafes/cargo-check.html
command cargo check --all-targets --all-features

# Check for linter errors
# https://github.com/rust-lang/rust-clippy
command cargo clippy --all-targets --all-features -- -D warnings


# Check code formatting
# https://github.com/rust-lang/rustfmt
command cargo fmt --all --check
