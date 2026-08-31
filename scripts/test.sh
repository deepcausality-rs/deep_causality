#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
set -o errexit
set -o nounset
set -o pipefail


# MLX GPU acceleration can only be tested single threaded
# because of its single command quqeue that is not threat safe.
command cargo test --doc --all-features
command cargo test --all-features -- --test-threads=1