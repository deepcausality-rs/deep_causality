#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# `alloc` is a level, not a complete configuration: it grants heap support and selects no
# float-math backend. So a crate built with `--no-default-features --features alloc`, for a
# target without `std`, must do one of two things:
#
#   - build, when nothing below it needs float math, or
#   - stop at the guard in deep_causality_num, with deep_causality_num the only crate that fails.
#
# Anything else fails the check: a `std` leak, an `alloc` level that is not forwarded to a
# dependency, a guard that went missing. `--keep-going` builds every crate that does not depend on
# the failed one, so the set of failures does not depend on job scheduling.
#
# Every workspace crate is accounted for: std-only crates are listed in bare_metal.sh, a no_std
# crate without an `alloc` feature is core-only and has no alloc level to check, and a crate that
# is neither fails the check.
#
# Usage: scripts/check_alloc.sh [target]    (default: thumbv7em-none-eabihf, 32-bit Cortex-M4F)
set -o errexit
set -o nounset
set -o pipefail

source "$(dirname "${BASH_SOURCE[0]}")/bare_metal.sh"
cd "$DC_REPO_ROOT"

TARGET="${1:-$DC_BARE_METAL_DEFAULT_TARGET}"
dc_require_target "$TARGET"

GUARD="deep_causality_num has no float-math backend"

failed=()
checked=0
dc_check_std_only_list || failed+=("(std-only list)")

for i in "${!DC_CRATES[@]}"; do
    c="${DC_CRATES[$i]}"
    dir="${DC_CRATE_DIRS[$i]}"
    reason="$(dc_std_only_reason "$c" || true)"
    if [ -n "$reason" ]; then
        echo "--- $c: std-only ($reason)"
        continue
    fi
    if ! dc_has_feature "$dir" alloc; then
        if dc_has_feature "$dir" no-std; then
            echo "--- $c: core-only, no alloc level"
        else
            echo "!! $c declares neither alloc nor no-std and is not listed as std-only in bare_metal.sh"
            failed+=("$c")
        fi
        continue
    fi

    echo "==> $c alloc-only ($TARGET)"
    checked=$((checked + 1))
    if out="$(cargo build --keep-going -p "$c" --lib --no-default-features --features alloc \
        --target "$TARGET" 2>&1)"; then
        echo "    builds"
        continue
    fi
    broken="$(echo "$out" | sed -n 's/^error: could not compile `\([^`]*\)`.*/\1/p' | sort -u)"
    if [ "$broken" = "deep_causality_num" ] && echo "$out" | grep -q "$GUARD"; then
        echo "    rejected by the deep_causality_num guard"
    else
        echo "$out"
        echo "!! $c: alloc-only build did not stop at the deep_causality_num guard; failed crates:" $broken
        failed+=("$c")
    fi
done

if [ "${#failed[@]}" -ne 0 ]; then
    echo "alloc check failed on $TARGET:"
    printf '  %s\n' "${failed[@]}"
    exit 1
fi

# An empty run would pass by checking nothing.
if [ "$checked" -eq 0 ]; then
    echo "No crate has an alloc feature; refusing to report success."
    exit 1
fi

echo "All $checked alloc crates build alloc-only or stop at the deep_causality_num guard on $TARGET."
