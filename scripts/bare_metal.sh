#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# Shared by check_no_std.sh and check_alloc.sh: the crates that are std-only by decision, and
# the helpers both checks use.
#
# ---------------------------------------------------------------------------------------------
# Every crate is checked or excluded by name
# ---------------------------------------------------------------------------------------------
#
# Selecting crates by feature key alone lets a crate that never declared `no-std` pass both
# checks by being skipped. So the checks walk every workspace crate (crates.sh). A crate without
# a `no-std` feature must be listed below with the reason, and a listed crate that declares
# `no-std` fails as a stale entry. Delete the line when a crate gains no_std support.
#
# Usage:
#
#   source "$(dirname "${BASH_SOURCE[0]}")/bare_metal.sh"

if [ "${DC_BARE_METAL_SOURCED:-}" = "1" ]; then
    return 0 2>/dev/null || true
fi

source "$(dirname "${BASH_SOURCE[0]}")/crates.sh"

DC_BARE_METAL_DEFAULT_TARGET="thumbv7em-none-eabihf"

# <crate>|<reason>. Reasons name what ties the crate to `std`.
DC_STD_ONLY=(
    "deep_causality|depends on deep_causality_context and deep_causality_uncertain; uses std HashMap and std::sync"
    "deep_causality_cfd|depends on deep_causality_topology and deep_causality_file; writes output files (std::fs, std::io)"
    "deep_causality_context|depends on deep_causality_uncertain; uses std HashMap and HashSet"
    "deep_causality_context_store|the in-memory reference backend in utils_test uses std::sync::Mutex"
    "deep_causality_discovery|reads CSV and Parquet files (std::fs, std::io, csv, parquet); depends on deep_causality_topology"
    "deep_causality_ethos|depends on deep_causality_context; uses std HashMap and HashSet"
    "deep_causality_file|filesystem loaders (std::fs, std::io); depends on chrono"
    "deep_causality_tempfile|creates and removes files and directories (std::fs, std::io)"
    "deep_causality_topology|uses std HashMap, HashSet and std::sync; Rayon-parallel loops"
    "deep_causality_uncertain|uses std HashMap and HashSet"
)

# Prints the reason when <crate> is listed as std-only; returns 1 otherwise.
dc_std_only_reason() {
    local entry
    for entry in "${DC_STD_ONLY[@]}"; do
        if [ "${entry%%|*}" = "$1" ]; then
            echo "${entry#*|}"
            return 0
        fi
    done
    return 1
}

# Succeeds when the `[features]` table of <crate dir>/Cargo.toml declares <feature>.
dc_has_feature() {
    awk -v key="$2" '
        /^\[features\]/ { in_f = 1; next }
        /^\[/           { in_f = 0 }
        in_f && $0 ~ "^" key "[[:space:]]*=" { found = 1 }
        END             { exit !found }
    ' "$1/Cargo.toml"
}

# Exits unless <target> is installed and has no operating system. A hosted target ships `std`,
# so a dependency that enables `std` links there without complaint and the check proves nothing.
# `target_os = "none"` is the condition deep_causality_par's `parallel` guard keys on as well.
dc_require_target() {
    if ! rustup target list --installed | grep -qx "$1"; then
        echo "Target $1 is not installed. Run: rustup target add $1"
        exit 1
    fi
    if ! rustc --print cfg --target "$1" | grep -qx 'target_os="none"'; then
        echo "Target $1 has an operating system and ships std; pass a bare-metal target such as" \
            "$DC_BARE_METAL_DEFAULT_TARGET."
        exit 1
    fi
}

# A listed name that is not a workspace crate is an exclusion that no longer does anything.
dc_check_std_only_list() {
    local entry name c found status=0
    for entry in "${DC_STD_ONLY[@]}"; do
        name="${entry%%|*}"
        found=0
        for c in "${DC_CRATES[@]}"; do
            [ "$c" = "$name" ] && found=1 && break
        done
        if [ "$found" -eq 0 ]; then
            echo "bare_metal.sh: '$name' is listed as std-only but is not a workspace crate"
            status=1
        fi
    done
    return "$status"
}

DC_BARE_METAL_SOURCED=1
