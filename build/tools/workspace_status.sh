#!/usr/bin/env bash
#
# Copyright (c) "2025" . Marvin Hansen All Rights Reserved.
#

set -o errexit
set -o nounset
set -o pipefail

echo "STABLE_GIT_COMMIT $(git rev-parse --short HEAD)"
