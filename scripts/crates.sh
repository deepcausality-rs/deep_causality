#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# The workspace's crate list, read from the root Cargo.toml.
#
# ---------------------------------------------------------------------------------------------
# Why this exists
# ---------------------------------------------------------------------------------------------
#
# Four scripts used to carry the same list by hand: sbom.sh, miri.sh, check.sh and format.sh. By
# the time this replaced them all four had drifted, and each was missing a *different* set:
#
#   sbom.sh    28 entries, missing deep_causality_linear, deep_causality_quantum
#   miri.sh    27 entries, missing deep_causality_linear, _num_rational, _quantum
#   check.sh   27 entries, missing deep_causality_linear, _num_rational, _quantum
#   format.sh  28 entries, missing deep_causality_linear, _num_rational
#
# Nothing failed when a crate was missing — the loop simply skipped it. A crate could be added to
# the workspace and go unformatted, unaudited, and shipped without an SBOM, and the only signal
# was its absence from a list nobody reads. Cargo.toml is the one place a crate cannot be missing
# from and still exist, so it is the one place to read.
#
# ---------------------------------------------------------------------------------------------
# Usage
# ---------------------------------------------------------------------------------------------
#
#   source "$(dirname "${BASH_SOURCE[0]}")/crates.sh"
#
#   for c in "${DC_CRATES[@]}"; do ... done      # package names
#   for d in "${DC_CRATE_DIRS[@]}"; do ... done  # directories, same order
#
#   for c in $(dc_crates_except deep_causality_cfd); do ... done
#
# ---------------------------------------------------------------------------------------------
# Exclusions are the caller's, and they are visible
# ---------------------------------------------------------------------------------------------
#
# This never filters. A script that must skip a crate says so at its own call site with
# `dc_crates_except`, next to the reason. That is the difference between an exclusion and an
# omission: the first is a decision someone can read and argue with, the second is what the four
# hand-maintained lists had.

# Idempotent: sourcing twice is a no-op rather than a redefinition.
if [ "${DC_CRATES_SOURCED:-}" = "1" ]; then
    return 0 2>/dev/null || true
fi

# The repo root, however this was sourced from.
if command -v git >/dev/null 2>&1 && git rev-parse --show-toplevel >/dev/null 2>&1; then
    DC_REPO_ROOT="$(git rev-parse --show-toplevel)"
else
    DC_REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi

if [ ! -f "$DC_REPO_ROOT/Cargo.toml" ]; then
    echo "crates.sh: no Cargo.toml at $DC_REPO_ROOT" >&2
    return 1 2>/dev/null || exit 1
fi

# The `members = [...]` array of the `[workspace]` table, one entry per line, globs unexpanded.
#
# Scoped to the `[workspace]` table so a `members` key in some other table cannot be picked up, and
# stops at the closing bracket so the rest of the file is not scanned.
dc__raw_members() {
    awk '
        /^\[workspace\]/            { in_ws = 1; next }
        /^\[/                       { in_ws = 0 }
        in_ws && /^[[:space:]]*members[[:space:]]*=/ { in_m = 1 }
        in_m                        { print }
        in_m && /\]/                { exit }
    ' "$DC_REPO_ROOT/Cargo.toml" |
        grep -o '"[^"]*"' |
        tr -d '"'
}

DC_CRATES=()
DC_CRATE_DIRS=()

dc__collect() {
    local pattern dir name
    # Globs are expanded against the repo root, and anything without a Cargo.toml is dropped —
    # `examples/*` matches LICENSE and README.md, which Cargo ignores and so does this.
    #
    # The example crates are dropped too. They are not published, not audited and not shipped, so
    # none of the consumers wants them; excluding them here keeps every call site free of the
    # filter.
    for pattern in $(dc__raw_members); do
        for dir in "$DC_REPO_ROOT"/$pattern; do
            [ -f "$dir/Cargo.toml" ] || continue
            name="$(sed -n 's/^name[[:space:]]*=[[:space:]]*"\(.*\)".*/\1/p' "$dir/Cargo.toml" | head -1)"
            [ -n "$name" ] || continue
            # Relative to the root, which is what every consumer wants to pass to a tool.
            dir="${dir#"$DC_REPO_ROOT"/}"
            case "$dir" in
                examples/*) continue ;;
            esac
            DC_CRATES+=("$name")
            DC_CRATE_DIRS+=("$dir")
        done
    done
}

dc__collect

# An empty list would make every consumer a silent no-op, which is the failure this file exists to
# prevent. Refuse instead.
if [ "${#DC_CRATES[@]}" -eq 0 ]; then
    echo "crates.sh: read no crates from $DC_REPO_ROOT/Cargo.toml — refusing to continue" >&2
    return 1 2>/dev/null || exit 1
fi

# The crate names, minus the ones named as arguments, one per line.
#
# Prints a warning if an argument matches nothing: a name that no longer exists is an exclusion
# that has quietly stopped doing anything, and that should be noticed rather than tolerated.
dc_crates_except() {
    local skip c matched
    for skip in "$@"; do
        matched=0
        for c in "${DC_CRATES[@]}"; do
            [ "$c" = "$skip" ] && matched=1 && break
        done
        [ "$matched" -eq 1 ] || echo "crates.sh: '$skip' is excluded but is not a workspace crate" >&2
    done
    for c in "${DC_CRATES[@]}"; do
        for skip in "$@"; do
            [ "$c" = "$skip" ] && continue 2
        done
        echo "$c"
    done
}

DC_CRATES_SOURCED=1
