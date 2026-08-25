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
source "$(dirname "${BASH_SOURCE[0]}")/crates.sh"
for CRATE_DIR in "${DC_CRATE_DIRS[@]}"; do
    cargo machete "$CRATE_DIR"/
done



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
