# SPDX-License-Identifier: MIT
# Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

# bin/bash
set -o errexit
set -o nounset
set -o pipefail

# https://github.com/taiki-e/cargo-llvm-cov
command cargo llvm-cov --open