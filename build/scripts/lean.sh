#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
set -o errexit
set -o nounset
set -o pipefail

# Run all LEAN proofs.
#
# The Lean formalization is a separate `lake` project under `lean/` (see lean/README.md).
# `lake build` compiles and machine-checks every proof module imported by the root
# aggregator `DeepCausalityFormal.lean`. A broken law fails the build — this is the same
# L1 gate CI enforces in .github/workflows/formalization.yml.

# Locate the lean/ directory relative to this script so it works whether the script is
# sourced (via `make lean`) or executed directly.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LEAN_DIR="${SCRIPT_DIR}/../../lean"

cd "${LEAN_DIR}"

# Download prebuilt Mathlib artifacts (fast; non-fatal if unavailable — build still checks proofs).
command lake exe cache get || true

# Compile and check all proofs.
command lake build

command echo "LEAN OK"
