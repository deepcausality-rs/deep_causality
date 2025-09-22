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
    "deep_causality_data_structures"
    "deep_causality_macros"
    "deep_causality_num"
    "deep_causality_rand"
    "deep_causality_tensor"
    "deep_causality_uncertain"
    "ultragraph"
)

for CRATE_NAME in "${CRATES[@]}"; do
    echo "Generating SBOM for crate: $CRATE_NAME"
    cargo sbom --cargo-package "$CRATE_NAME" --output-format=spdx_json_2_3 > "$CRATE_NAME"/sbom.spdx
    if [ $? -eq 0 ]; then
        echo "Successfully generated SBOM for $CRATE_NAME"
    else
        echo "Failed to generate SBOM for $CRATE_NAME"
    fi
done

echo "SBOM generation complete."


