#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
set -o errexit
set -o nounset
set -o pipefail


# command cargo fix --lib --allow-dirty

# command cargo clippy --fix --allow-dirty --all-targets -- -D warnings

# fix all configured features
command cargo fix --lib --examples --allow-dirty --all-targets --all-features

# Double check if nothing has beem missed
command cargo clippy --examples --all-targets --all-features -- -D warnings
