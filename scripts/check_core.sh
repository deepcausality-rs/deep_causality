#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# A `core`-only crate must link into an application that defines no `#[global_allocator]`.
# Building its rlib proves nothing: rustc demands an allocator only when it links a final
# artifact, and it demands one whenever the `alloc` crate is anywhere in the graph, whether or not
# anything allocates. A `no-std` feature that switches `alloc` on in some dependency passes every
# rlib build and fails in the application.
#
# The crates checked are those whose `no-std` feature does not name `alloc`. Each is linked into a
# throwaway `no_std` static library with no allocator, for a target without an operating system.
#
# Usage: scripts/check_core.sh [target]    (default: thumbv7em-none-eabihf, 32-bit Cortex-M4F)
set -o errexit
set -o nounset
set -o pipefail

source "$(dirname "${BASH_SOURCE[0]}")/bare_metal.sh"
cd "$DC_REPO_ROOT"

TARGET="${1:-$DC_BARE_METAL_DEFAULT_TARGET}"
dc_require_target "$TARGET"

probe="$(mktemp -d)"
trap 'rm -rf "$probe"' EXIT
# Shared across probes and runs, so each crate's dependencies build once.
export CARGO_TARGET_DIR="$DC_REPO_ROOT/target/core-probe"

failed=()
linked=0

for i in "${!DC_CRATES[@]}"; do
    c="${DC_CRATES[$i]}"
    dir="${DC_CRATE_DIRS[$i]}"
    dc_has_feature "$dir" no-std || continue
    if ! dc_no_std_is_core "$dir"; then
        echo "--- $c: its no-std feature enables alloc"
        continue
    fi

    cat >"$probe/Cargo.toml" <<EOF
[package]
name = "core_probe"
version = "0.0.0"
edition = "2024"

[lib]
crate-type = ["staticlib"]
path = "lib.rs"

[dependencies]
$c = { path = "$DC_REPO_ROOT/$dir", default-features = false, features = ["no-std"] }

[profile.dev]
panic = "abort"

[workspace]
EOF
    cat >"$probe/lib.rs" <<EOF
#![no_std]
extern crate $c as _;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
EOF

    echo "==> $c links without an allocator ($TARGET)"
    linked=$((linked + 1))
    if ! cargo build --quiet --manifest-path "$probe/Cargo.toml" --target "$TARGET"; then
        failed+=("$c")
    fi
done

if [ "${#failed[@]}" -ne 0 ]; then
    echo "core check failed on $TARGET; these need a #[global_allocator]:"
    printf '  %s\n' "${failed[@]}"
    exit 1
fi

# An empty run would pass by checking nothing.
if [ "$linked" -eq 0 ]; then
    echo "No crate has a core-only no-std feature; refusing to report success."
    exit 1
fi

echo "All $linked core-only crates link without an allocator on $TARGET."
