#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#

set -o errexit
set -o nounset
set -o pipefail

CRATES=(
    "deep_causality"
    "deep_causality_algorithms"
    "deep_causality_ast"
    "deep_causality_calculus"
    "deep_causality_cfd"
    "deep_causality_core"
    "deep_causality_data_structures"
    "deep_causality_discovery"
    "deep_causality_ethos"
    "deep_causality_fft"
    "deep_causality_haft"
    "deep_causality_macros"
    "deep_causality_metric"
    "deep_causality_multivector"
    "deep_causality_num"
    "deep_causality_par"
    "deep_causality_rand"
    "deep_causality_physics"
    "deep_causality_sparse"
    "deep_causality_tensor"
    "deep_causality_topology"
    "deep_causality_uncertain"
    "ultragraph"
)

for CRATE_NAME in "${CRATES[@]}"; do
    echo "Running MIRI for crate: $CRATE_NAME"
     if !  cargo miri test -p "$CRATE_NAME" --lib --tests
    then
        echo "Failed to run MIRI for $CRATE_NAME"
    fi
done

echo "MIRI complete."