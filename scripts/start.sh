# SPDX-License-Identifier: MIT
# Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
set -o errexit
set -o nounset
set -o pipefail


command echo ""
command echo "Checking for rustup update"
command rustup upgrade


command echo ""
command echo "Checking for rustc update"
command rustup update stable


command echo ""
command echo "Running git pull to update local repo"
command git pull


command echo ""
command echo "Build project"
command command cargo build