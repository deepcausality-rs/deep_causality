# Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

# bin/bash
set -o errexit
set -o nounset
set -o pipefail

# https://github.com/taiki-e/cargo-llvm-cov
command cargo llvm-cov --open