# Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
set -o errexit
set -o nounset
set -o pipefail

command  repolinter lint -g https://github.com/deepcausality-rs/deep_causality.git --format markdown > docs/LF_Repo_Lint.md
