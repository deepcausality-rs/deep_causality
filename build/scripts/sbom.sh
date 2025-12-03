#
# SPDX-License-Identifier: MIT
# Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
#

set -o errexit
set -o nounset
set -o pipefail

CRATES=(
    "deep_causality"
    "deep_causality_algorithms"
    "deep_causality_ast"
    "deep_causality_core"
    "deep_causality_data_structures"
    "deep_causality_discovery"
    "deep_causality_haft"
    "deep_causality_macros"
    "deep_causality_multivector"
    "deep_causality_num"
    "deep_causality_rand"
    "deep_causality_sparse"
    "deep_causality_tensor"
    "deep_causality_topology"
    "deep_causality_uncertain"
    "ultragraph"
)

for CRATE_NAME in "${CRATES[@]}"; do
    echo "Generating SBOM for crate: $CRATE_NAME"

    if ! cargo sbom --cargo-package "$CRATE_NAME" --output-format=spdx_json_2_3 > "$CRATE_NAME"/"$CRATE_NAME"_sbom.spdx.json
    then
        echo "Failed to generate SBOM for $CRATE_NAME"
    fi

     if ! sha256sum "$CRATE_NAME"/"$CRATE_NAME"_sbom.spdx.json > "$CRATE_NAME"/"$CRATE_NAME"_sbom.spdx.json.sha
     then
        echo "Failed to generate HASH over SBOM for $CRATE_NAME"
     fi

done

echo "SBOM generation complete."
