#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# Every crate that declares a `no-std` feature must build for a target that has no standard
# library. Building with `--features no-std` on the host proves nothing: the host has `std`, so a
# dependency that enables `std` links it without complaint. Only a target without `std` refuses.
#
# Every workspace crate is accounted for: it declares `no-std` and is built, or it is listed as
# std-only in bare_metal.sh with the reason. A crate that is neither fails the check.
#
# A crate whose `no-std` builds on `core` alone and that declares `alloc` as an add-on is built a
# second time with `no-std,alloc`, so the parts it gates behind `alloc` build for the target too.
#
# Each crate builds in its own `cargo` call. One call over several `-p` flags unifies their
# features, so a sibling that enables `std` would mask the crate that needs checking.
#
# Usage: scripts/check_no_std.sh [target]    (default: thumbv7em-none-eabihf, 32-bit Cortex-M4F)
set -o errexit
set -o nounset
set -o pipefail

source "$(dirname "${BASH_SOURCE[0]}")/bare_metal.sh"
cd "$DC_REPO_ROOT"

TARGET="${1:-$DC_BARE_METAL_DEFAULT_TARGET}"
dc_require_target "$TARGET"

failed=()
built=0
dc_check_std_only_list || failed+=("(std-only list)")

for i in "${!DC_CRATES[@]}"; do
    c="${DC_CRATES[$i]}"
    reason="$(dc_std_only_reason "$c" || true)"
    if dc_has_feature "${DC_CRATE_DIRS[$i]}" no-std; then
        if [ -n "$reason" ]; then
            echo "!! $c declares no-std but is listed as std-only in bare_metal.sh; delete the entry"
            failed+=("$c")
            continue
        fi
        echo "==> $c ($TARGET)"
        if cargo build -p "$c" --lib --no-default-features --features no-std --target "$TARGET"; then
            built=$((built + 1))
        else
            failed+=("$c")
        fi
        if dc_no_std_is_core "${DC_CRATE_DIRS[$i]}" && dc_has_feature "${DC_CRATE_DIRS[$i]}" alloc; then
            echo "==> $c with alloc ($TARGET)"
            if ! cargo build -p "$c" --lib --no-default-features --features no-std,alloc \
                --target "$TARGET"; then
                failed+=("$c (no-std,alloc)")
            fi
        fi
    elif [ -n "$reason" ]; then
        echo "--- $c: std-only ($reason)"
    else
        echo "!! $c declares no no-std feature and is not listed as std-only in bare_metal.sh"
        failed+=("$c")
    fi
done

if [ "${#failed[@]}" -ne 0 ]; then
    echo "no_std check failed on $TARGET:"
    printf '  %s\n' "${failed[@]}"
    exit 1
fi

# An empty run would pass by checking nothing.
if [ "$built" -eq 0 ]; then
    echo "No crate was built; refusing to report success."
    exit 1
fi

echo "All $built no-std crates build for $TARGET; ${#DC_STD_ONLY[@]} crates are std-only."
