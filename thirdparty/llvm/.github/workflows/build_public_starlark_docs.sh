#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail -x

if [[ $# -lt 1 ]]; then
  echo "Usage: $(basename "$0") <targets_file> [output_base]" >&2
  exit 1
fi

TARGETS_FILE=$1
OUTPUT_BASE="${2:-}"

common_args=()
if [[ -n "${OUTPUT_BASE}" ]]; then
  common_args+=(--output_base="${OUTPUT_BASE}")
fi

bazel "${common_args[@]}" query --output=label --output_file="${TARGETS_FILE}" 'kind("starlark_doc_extract rule", //config/... + //constraints/... + //extensions/... + //platforms/... + //toolchain/...)'
bazel "${common_args[@]}" build --target_pattern_file="${TARGETS_FILE}" --nocheck_visibility
