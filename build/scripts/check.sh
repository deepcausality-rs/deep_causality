#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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
cargo machete deep_causality/
cargo machete deep_causality_algorithms/
cargo machete deep_causality_ast/
cargo machete deep_causality_core/
cargo machete deep_causality_data_structures/
cargo machete deep_causality_discovery/
cargo machete deep_causality_effects/
cargo machete deep_causality_ethos/
cargo machete deep_causality_haft/
cargo machete deep_causality_macros/
cargo machete deep_causality_metric/
cargo machete deep_causality_multivector/
cargo machete deep_causality_num/
cargo machete deep_causality_physics/
cargo machete deep_causality_rand/
cargo machete deep_causality_sparse/
cargo machete deep_causality_tensor/
cargo machete deep_causality_topology/
cargo machete deep_causality_uncertain/
cargo machete examples/
cargo machete ultragraph/


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
