# Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

# bin/bash
set -o errexit
set -o nounset
set -o pipefail

# Check for outdated dependencies
# https://github.com/kbknapp/cargo-outdated
command cargo outdated


# Scan for unused dependencies
# https://crates.io/crates/cargo-udeps
command cargo +nightly udeps


# Scan again to report all unfixed vulnerabilities
# https://crates.io/crates/cargo-audi
command cargo audit


command cargo check

# Run linter
# https://github.com/rust-lang/rust-clippy
command cargo clippy