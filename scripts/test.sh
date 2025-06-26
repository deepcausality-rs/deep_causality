#
# SPDX-License-Identifier: MIT
# Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
#
set -o errexit
set -o nounset
set -o pipefail


# https://nexte.st/book/installing-from-source.html
# cargo install cargo-nextest --locked

FEATURES=unsafe RUSTFLAGS='-C target-cpu=native' cargo test --doc --release

# https://llogiq.github.io/2017/06/01/perf-pitfalls.html
FEATURES=unsafe RUSTFLAGS='-C target-cpu=native' cargo nextest run --release