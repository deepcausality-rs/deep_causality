#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# Every crate that declares a `no-std` feature must build for a target that has no standard
# library. Building with `--features no-std` on the host proves nothing: the host has `std`, so a
# dependency that enables `std` links it without complaint. Only a target without `std` refuses.
#
# The crate list is the workspace crates (crates.sh) whose `[features]` table has a `no-std` key.
# A crate that adds the feature is checked from then on; no list here needs an edit.
#
# Each crate builds in its own `cargo` call. One call over several `-p` flags unifies their
# features, so a sibling that enables `std` would mask the crate that needs checking.
#
# Usage: scripts/check_no_std.sh [target]    (default: thumbv7em-none-eabihf, 32-bit Cortex-M4F)
set -o errexit
set -o nounset
set -o pipefail

source "$(dirname "${BASH_SOURCE[0]}")/crates.sh"
cd "$DC_REPO_ROOT"

TARGET="${1:-thumbv7em-none-eabihf}"

if ! rustup target list --installed | grep -qx "$TARGET"; then
    echo "Target $TARGET is not installed. Run: rustup target add $TARGET"
    exit 1
fi

NO_STD_CRATES=()
for i in "${!DC_CRATES[@]}"; do
    if awk '
        /^\[features\]/ { in_f = 1; next }
        /^\[/           { in_f = 0 }
        in_f && /^no-std[[:space:]]*=/ { found = 1 }
        END             { exit !found }
    ' "${DC_CRATE_DIRS[$i]}/Cargo.toml"; then
        NO_STD_CRATES+=("${DC_CRATES[$i]}")
    fi
done

# An empty list would pass by checking nothing.
if [ "${#NO_STD_CRATES[@]}" -eq 0 ]; then
    echo "No workspace crate declares a no-std feature; refusing to report success."
    exit 1
fi

failed=()
for c in "${NO_STD_CRATES[@]}"; do
    echo "==> $c ($TARGET)"
    if ! cargo build -p "$c" --lib --no-default-features --features no-std --target "$TARGET"; then
        failed+=("$c")
    fi
done

if [ "${#failed[@]}" -ne 0 ]; then
    echo "no_std build failed for ${#failed[@]} of ${#NO_STD_CRATES[@]} crates on $TARGET:"
    printf '  %s\n' "${failed[@]}"
    exit 1
fi

echo "All ${#NO_STD_CRATES[@]} no-std crates build for $TARGET."
