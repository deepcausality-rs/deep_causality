#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# `alloc` is a level, not a complete configuration: it grants heap support and selects no
# float-math backend. So a crate built with `--no-default-features --features alloc`, for a
# target without `std`, must do one of two things:
#
#   - build, when nothing below it needs float math, or
#   - stop at the guard in deep_causality_num.
#
# The second case is read from cargo's JSON messages, not from rendered text. A rejected build must
# fail, deep_causality_num must be the only compile target that emits an error, and its first error
# must carry the `[no-float-backend]` tag of the guard's `compile_error!`. rustc emits that error
# during macro expansion, before the missing-backend errors that follow it in the same crate.
#
# Anything else fails the check: a `std` leak, an `alloc` level that is not forwarded to a
# dependency, a guard that went missing, a failure cargo reports outside the compiler.
# `--keep-going` builds every crate that does not depend on the failed one, so the set of failures
# does not depend on job scheduling.
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

GUARD_TAG="[no-float-backend]"

command -v jq >/dev/null || { echo "check_alloc.sh needs jq to read cargo's JSON messages"; exit 1; }

stderr_log="$(mktemp)"
trap 'rm -f "$stderr_log"' EXIT

# The compile errors in a stream of cargo JSON messages, one `<target>\t<message>` line each, in
# the order rustc emitted them.
compile_errors() {
    jq -r 'select(.reason == "compiler-message" and .message.level == "error")
           | [.target.name, .message.message] | @tsv'
}

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
    if json="$(cargo build --message-format=json --keep-going -p "$c" --lib \
        --no-default-features --features alloc --target "$TARGET" 2>"$stderr_log")"; then
        echo "    builds"
        continue
    fi
    errors="$(echo "$json" | compile_errors)"
    broken="$(echo "$errors" | cut -f1 | sed '/^$/d' | sort -u)"
    first="$(echo "$errors" | head -1 | cut -f2)"
    if [ "$broken" = "deep_causality_num" ] && [[ "$first" == "$GUARD_TAG"* ]]; then
        echo "    rejected by the deep_causality_num guard"
    else
        echo "$json" | jq -r 'select(.reason == "compiler-message" and .message.level == "error")
                              | .message.rendered'
        cat "$stderr_log"
        echo "!! $c: alloc-only build did not stop at the deep_causality_num guard;" \
            "targets with errors:" ${broken:-none}
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
