# SPDX-License-Identifier: MIT
# Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
set -o errexit
set -o nounset
set -o pipefail

command  repolinter lint -g https://github.com/deepcausality-rs/deep_causality.git --format markdown > docs/LF_Repo_Lint.md
