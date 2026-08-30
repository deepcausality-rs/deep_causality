#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# Every Cargo `[[example]]` must have a matching Bazel target, so an example added to Cargo
# cannot silently go unbuilt under Bazel. Two places declare them: the example packages under
# `examples/`, and the library crates that carry their own verification harnesses and studies
# (deep_causality_cfd, deep_causality_algorithms, deep_causality_haft, and others).
#
# Examples that are Cargo-only by decision are listed below; the reason lives in the owning
# package's BUILD.bazel next to where the target would have been.
set -o errexit
set -o nounset
set -o pipefail

# The library-crate list comes from the workspace manifest via crates.sh. It replaces a
# `*/Cargo.toml` glob, which reached exactly one level down and therefore stopped seeing every
# crate under `deep_causality_unified_math/` on the day they moved there. Twelve
# `deep_causality_haft` examples went unchecked while this script still printed
# "All Cargo examples have a Bazel target" and exited 0. A glob that encodes the repository's
# shape is a coverage cliff waiting for the next move; the manifest cannot be wrong about which
# crates exist.
source "$(dirname "${BASH_SOURCE[0]}")/crates.sh"
cd "$DC_REPO_ROOT"

# <package>:<example name> pairs that Bazel deliberately does not build.
CARGO_ONLY="causal_discovery_examples:example_ml_rca"

status=0
scanned=0

# crates.sh drops `examples/*` on purpose, so those are added back here.
MANIFESTS=""
for d in examples/*/; do
    [ -f "${d}Cargo.toml" ] && MANIFESTS="$MANIFESTS ${d}Cargo.toml"
done
for d in "${DC_CRATE_DIRS[@]}"; do
    [ -f "$d/Cargo.toml" ] && MANIFESTS="$MANIFESTS $d/Cargo.toml"
done

for manifest in $MANIFESTS; do
    dir=$(dirname "$manifest")
    package=$(basename "$dir")
    scanned=$((scanned + 1))

    cargo_targets=$(grep -A1 '^\[\[example\]\]' "$manifest" |
        sed -n 's/^name = "\(.*\)"$/\1/p' | sort || true)

    # Under examples/, a package with no `[[example]]` sections builds `src/main.rs` as one
    # binary named after the package (starter_example, tokio_example). Library crates have no
    # such fallback: a crate with no `[[example]]` simply has nothing to check.
    if [ -z "$cargo_targets" ]; then
        if [ "$dir" != "${dir#examples/}" ] && [ -f "$dir/src/main.rs" ]; then
            cargo_targets="$package"
        else
            continue
        fi
    fi

    bazel_targets=$(bazel query "kind(rust_binary, //$dir:all)" 2>/dev/null |
        sed 's|.*:||' | sort || true)

    for target in $cargo_targets; do
        if echo "$CARGO_ONLY" | grep -qw "$package:$target"; then
            continue
        fi
        if ! echo "$bazel_targets" | grep -qx "$target"; then
            echo "MISSING Bazel target: //$dir:$target"
            status=1
        fi
    done
done

if [ "$status" -eq 0 ]; then
    echo "All Cargo examples have a Bazel target ($scanned manifests scanned)."
fi

exit "$status"
