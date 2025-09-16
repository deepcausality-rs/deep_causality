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
FEATURES=unsafe cargo outdated --workspace

FEATURES=unsafe cargo machete deep_causality deep_causality_algorithms deep_causality_rand deep_causality_data_structures deep_causality_macros deep_causality_uncertain ultragraph

# Scan for unused dependencies
# https://crates.io/crates/cargo-udeps
FEATURES=unsafe cargo +nightly udeps --all-targets


# Scan again to report all unfixed vulnerabilities
# https://crates.io/crates/cargo-audit
#FEATURES=unsafe cargo audit


# Check a package and all of its dependencies for errors.
# https://doc.rust-lang.org/cargo/FEATURES=unsafes/cargo-check.html
FEATURES=unsafe cargo check --all-targets

# Consider checking each crate for re-exporting external types
# https://crates.io/crates/cargo-check-external-types
# cargo +nightly check-external-types


# Check for linter errors
# https://github.com/rust-lang/rust-clippy
FEATURES=unsafe cargo clippy --all-targets


# Check code formatting
# https://github.com/rust-lang/rustfmt
FEATURES=unsafe cargo fmt --all --check
