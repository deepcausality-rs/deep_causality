# SPDX-License-Identifier: MIT
# Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
set -o errexit
set -o nounset
set -o pipefail

#command cargo test --doc

# https://nexte.st/book/installing-from-source.html
# cargo install cargo-nextest --locked
command cargo nextest run