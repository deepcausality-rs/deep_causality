#!/usr/bin/env bash
# Effective (non-blank, non-comment) line counts for the reuse measurement.
cd "$(dirname "$0")" || exit 1

count() { grep -vcE '^[[:space:]]*(//|/\*|\*/?|$)' "$1"; }

# Count only lines between the first `impl <Trait> for` and the closing brace
# at column 0, for a given trait name in a file.
impl_loc() {
  awk -v t="$2" '
    $0 ~ "^impl.* " t " for " { inb=1 }
    inb { if ($0 !~ /^[[:space:]]*(\/\/|$)/) n++ }
    inb && /^}$/ { inb=0 }
    END { print n+0 }' "$1"
}

echo "== SHARED (written once in linear) =="
echo "  trait declarations  linear_b/src/lib.rs        : $(count linear_b/src/lib.rs)"
echo "  generic algorithms  linear_b/src/algorithms.rs : $(count linear_b/src/algorithms.rs)"
echo "  [Vec<F>] impl       linear_b/src/rows_of_rows.rs: $(count linear_b/src/rows_of_rows.rs)"
echo
echo "== PER-IMPLEMENTATION (written once per representation) =="
for pair in "consumer_b/src/dense.rs" "consumer_b/src/packed_gf2.rs" \
            "../deep_causality_tensor/src/types/causal_tensor/linear_seam.rs"; do
  v=$(impl_loc "$pair" MatrixView)
  r=$(impl_loc "$pair" RowOps)
  b=$(impl_loc "$pair" MatrixBuild)
  echo "  $pair"
  echo "     MatrixView=$v  RowOps=$r  MatrixBuild=$b  total=$((v + r + b))"
done
echo
echo "== DESIGN A / C =="
echo "  linear_a Matrix + rref : $(count linear_a/src/lib.rs)"
echo "  linear_a Gf2 scalar    : $(count linear_a/src/gf2_scalar.rs)"
echo "  linear_c free functions: $(count linear_c/src/lib.rs)"
