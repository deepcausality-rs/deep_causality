#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# Precision is a parameter of the program, not an assumption baked into call sites. An example
# that declares `type FloatType = ...` is stating that its arithmetic follows that one name, so
# this script flips the name through every shipped real field and rebuilds.
#
# It exists because the claim drifted silently. Six examples declared the alias and could not
# compile at `Float106`: a `const` holding a primitive literal (a `const` cannot hold a software
# scalar), an inherent `.sqrt()` or `.powi()` called without `deep_causality_algebra::Real` or
# `deep_causality_num::Float` in scope, an accumulator seeded with `0.0` that inferred `f64`, and
# a `SimplicialManifold<f64, FloatType>` that pinned the complex's coefficients regardless of the
# alias. Every one of them built at `f64` and the suite was green, so nothing objected.
#
# `f32` failures are reported separately, because a `From<f64>` bound anywhere in the call graph
# excludes `f32` alone: `f32` is the one shipped scalar with no `From<f64>` impl, which is why
# `deep_causality_num::lift` exists over `FromPrimitive` instead.
set -o errexit
set -o nounset
set -o pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

PACKAGE="${1:-mathematics_examples}"
MANIFEST="examples/$PACKAGE/Cargo.toml"
[ -f "$MANIFEST" ] || { echo "no manifest at $MANIFEST"; exit 1; }

TARGETS="deep_causality_num::Float106 f32 f64"

# <example name>:<precision> pairs that are known not to build, each with the reason. These are
# library bounds rather than example defects; the reason also lives in the example's alias
# docstring. Remove the entry when the bound is fixed, and the gate starts enforcing it again.
# Empty: nothing is currently excluded.
EXCLUDED=""

status=0
checked=0
skipped=0

# Pair each `[[example]]` name with its path, so a failure names the target to rerun.
while IFS='|' read -r name path; do
    src="examples/$PACKAGE/$path"
    [ -f "$src" ] || continue

    original=$(sed -n 's/^\(pub \)\{0,1\}type FloatType = \(.*\);$/\2/p' "$src" | head -1)
    if [ -z "$original" ]; then
        skipped=$((skipped + 1))
        continue
    fi
    checked=$((checked + 1))

    failures=""
    for t in $TARGETS; do
        if echo "$EXCLUDED" | grep -qw "$name:$t"; then
            continue
        fi
        sed -i.bak "s|^\(pub \)\{0,1\}type FloatType = .*;|\1type FloatType = $t;|" "$src"
        if ! cargo check -q -p "$PACKAGE" --example "$name" >/dev/null 2>&1; then
            failures="$failures $t"
        fi
        mv "$src.bak" "$src"
    done

    if [ -n "$failures" ]; then
        echo "FAILS at:$failures  --example $name  ($path)"
        status=1
    fi
done < <(awk '
    /^\[\[example\]\]/ { n=""; p=""; next }
    /^name *= *"/ { gsub(/^name *= *"|"$/, ""); n=$0; next }
    /^path *= *"/ { gsub(/^path *= *"|"$/, ""); p=$0; if (n != "" && p != "") print n "|" p; n=""; p="" }
' "$MANIFEST")

if [ "$status" -eq 0 ]; then
    echo "All $checked aliased examples in $PACKAGE build at every shipped precision."
    echo "($skipped examples declare no FloatType alias and were not checked.)"
fi

exit "$status"
