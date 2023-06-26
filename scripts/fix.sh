# Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
set -o errexit
set -o nounset
set -o pipefail

command cargo fix --lib --allow-dirty

command cargo clippy --fix --allow-dirty