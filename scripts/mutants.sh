#!/usr/bin/env bash
#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# Mutation testing: does the suite notice when the source is wrong?
#
# ---------------------------------------------------------------------------------------------
# What this catches that coverage does not
# ---------------------------------------------------------------------------------------------
#
# Coverage reports whether a line ran. A line can run and still be untested: assert a quantity
# that is zero for every input under test and any implementation of it passes. That is how the
# topological-charge normalization in `deep_causality_topology` stayed eight times too small
# through 513 executions of the line that computed it.
#
# `cargo mutants` changes one expression at a time and re-runs the tests. A mutant the tests do
# not object to is a decision nothing pins.
#
# ---------------------------------------------------------------------------------------------
# Usage
# ---------------------------------------------------------------------------------------------
#
#   scripts/mutants.sh                       # the default set, below
#   scripts/mutants.sh deep_causality_linear # one crate, all files
#   scripts/mutants.sh deep_causality_linear src/algorithms/kernels.rs
#
# Equivalent mutants are excluded in //.cargo/mutants.toml, each with its reason. A survivor not
# listed there is a real gap.
#
# ---------------------------------------------------------------------------------------------
# Why this is not run over the whole workspace
# ---------------------------------------------------------------------------------------------
#
# Every mutant costs a build plus a full test run for its crate. `deep_causality_linear`'s
# `algorithms/` alone yields 1156 of them. This is a tool to point at code where a wrong constant
# or a flipped comparison would produce a plausible answer rather than a crash: numeric kernels,
# tolerance gates, index arithmetic. Pointing it at a whole workspace costs hours and mostly
# re-confirms that error paths are tested.

set -o errexit
set -o nounset
set -o pipefail

if ! command -v cargo-mutants >/dev/null 2>&1; then
    echo "cargo-mutants is not installed. Install it with:" >&2
    echo "    cargo install cargo-mutants --locked" >&2
    exit 1
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

CRATE="${1:-}"
FILE="${2:-}"

# A package name is not its directory. The sixteen mathematics crates live under
# `deep_causality_unified_math/`, so `--file` needs the manifest directory rather than the package
# name. Cargo is the only thing that knows the mapping, so ask it.
crate_dir() {
    cargo metadata --no-deps --format-version 1 |
        tr ',' '\n' |
        grep -A0 "\"manifest_path\":[^,]*/$1/Cargo.toml" |
        sed -E 's|.*"manifest_path":"||; s|/Cargo.toml"?||' |
        sed "s|^$REPO_ROOT/||" |
        head -1
}

# The default target: the numeric algorithms, where a flipped comparison returns a plausible
# number instead of failing. Extend this list as other crates earn it.
if [ -z "$CRATE" ]; then
    echo "Mutation testing deep_causality_unified_math/deep_causality_linear/src/algorithms (the default set)"
    exec cargo mutants -p deep_causality_linear \
        --file 'deep_causality_unified_math/deep_causality_linear/src/algorithms/*.rs' \
        -j 8
fi

if [ -n "$FILE" ]; then
    DIR="$(crate_dir "$CRATE")"
    if [ -z "$DIR" ]; then
        echo "mutants.sh: no workspace package named $CRATE" >&2
        exit 1
    fi
    exec cargo mutants -p "$CRATE" --file "$DIR/$FILE" -j 8
fi

exec cargo mutants -p "$CRATE" -j 8
