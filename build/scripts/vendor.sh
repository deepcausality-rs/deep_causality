#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#

# bin/sh
set -o errexit
set -o nounset
set -o pipefail

# Updates all vendored crates
command bazel run //thirdparty:crates_vendor