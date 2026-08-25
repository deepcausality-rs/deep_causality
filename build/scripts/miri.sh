#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#

set -o errexit
set -o nounset
set -o pipefail

source "$(dirname "${BASH_SOURCE[0]}")/crates.sh"

# deep_causality_cfd is skipped deliberately. It is compute-bound -- 663 solver tests take about
# 127s natively and hours under Miri's interpreter -- and it contains no `unsafe`, so Miri has
# nothing there to find. This is an exclusion rather than an omission: delete the argument to put
# it back.
for CRATE_NAME in $(dc_crates_except deep_causality_cfd); do
    echo "Running MIRI for crate: $CRATE_NAME"
     if !  cargo miri test -p "$CRATE_NAME" --lib --tests
    then
        echo "Failed to run MIRI for $CRATE_NAME"
    fi
done

echo "MIRI complete."
