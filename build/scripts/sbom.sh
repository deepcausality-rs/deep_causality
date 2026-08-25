#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#

set -o errexit
set -o nounset
set -o pipefail

# The crate list comes from the workspace, not from a copy kept here.
source "$(dirname "${BASH_SOURCE[0]}")/crates.sh"

for CRATE_NAME in "${DC_CRATES[@]}"; do
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

echo "SBOM generation complete for ${#DC_CRATES[@]} crates."
