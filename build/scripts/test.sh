#
# SPDX-License-Identifier: MIT
# Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
#
set -o errexit
set -o nounset
set -o pipefail


cargo test --doc

cargo test

# MLX GPU acceleration can only be tested single threaded
# because of its single command quqeue that is not threat safe.
cargo test --doc  --features mlx
cargo test --features mlx -- --test-threads=1