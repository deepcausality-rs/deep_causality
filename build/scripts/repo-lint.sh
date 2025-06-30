#
# SPDX-License-Identifier: MIT
# Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
#
set -o errexit
set -o nounset
set -o pipefail

command  repolinter lint -g https://github.com/deepcausality-rs/deep_causality.git --format markdown > docs/LF_Repo_Lint.md
