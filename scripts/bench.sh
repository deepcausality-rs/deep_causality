# SPDX-License-Identifier: MIT
# Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

# bin/bash
set -o errexit
set -o nounset
set -o pipefail

#
# https://users.rust-lang.org/t/how-to-best-ensure-target-cpu-native/53167

FEATURES=unsafe RUSTFLAGS='-C target-cpu=native' cargo bench