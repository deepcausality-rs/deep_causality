#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#

set -o errexit
set -o nounset
set -o pipefail

# The crate list comes from the workspace, not from a copy kept here.
source "$(dirname "${BASH_SOURCE[0]}")/crates.sh"
cd "$DC_REPO_ROOT"

# A package name is not a directory. `--cargo-package` wants the name; the output file wants the
# path, and the two stopped being the same string when the mathematics crates moved under
# `deep_causality_unified_math/`. The arrays are index-aligned by crates.sh, so walk them together.
status=0

for i in "${!DC_CRATES[@]}"; do
    CRATE_NAME="${DC_CRATES[$i]}"
    CRATE_DIR="${DC_CRATE_DIRS[$i]}"
    SBOM="$CRATE_DIR/${CRATE_NAME}_sbom.spdx.json"
    echo "Generating SBOM for crate: $CRATE_NAME ($CRATE_DIR)"

    if ! cargo sbom --cargo-package "$CRATE_NAME" --output-format=spdx_json_2_3 > "$SBOM"
    then
        echo "Failed to generate SBOM for $CRATE_NAME"
        status=1
        continue
    fi

    if ! sha256sum "$SBOM" > "$SBOM.sha"
    then
        echo "Failed to generate HASH over SBOM for $CRATE_NAME"
        status=1
    fi
done

echo "SBOM generation complete for ${#DC_CRATES[@]} crates."
exit "$status"
