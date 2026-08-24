<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Scenario map

Task 2.3. Every scenario in the four capabilities the change names, against the test that serves it.
A scenario with no test is a requirement nobody checks, and it is invisible unless the mapping is
written down.

Status key: **T** covered by a test here · **P** covered by a compile probe recorded in
`openspec/notes/linear/BOUND-LEDGER.md` · **5** deferred to phase 5, where the consumer exists

## linear-matrix-representations (13)

| scenario | test |
|---|---|
| All three are reachable from the crate root | T `types::*::algebra_tests` construct each from the root |
| Sparse behaviour is preserved | 5 — needs the moved implementation and its ported suite |
| Sparse to dense round-trips | T `conversions_tests::test_sparse_to_dense_and_back_round_trips_including_the_shape` |
| Packing rejects a non-binary entry | T `conversions_tests::test_packing_rejects_an_entry_outside_zero_one_and_names_the_position` |
| Packing accepts the boundary-operator alphabet | T `conversions_tests::test_packing_accepts_the_boundary_operator_alphabet_by_reducing`, `packed_gf2::mod_tests::test_the_boundary_operator_alphabet_reduces_mod_two` |
| Two widths agree | T `gf2_tests::test_two_word_widths_agree_on_rank`, `packed_gf2::mod_tests::test_the_same_matrix_at_two_word_widths_reads_identically` |
| A column count that is not a multiple of the word width | T `packed_gf2::mod_tests::test_a_column_count_that_is_not_a_multiple_of_the_word_width`, `gf2_tests::test_a_column_count_past_a_word_boundary_does_not_change_the_rank` |
| Allocation matches the bit count | T `packed_gf2::mod_tests::test_allocation_matches_the_bit_count` |
| A non-square input to a square-only operation | T `elimination_tests::test_determinant_rejects_a_non_square_matrix` |
| The dimension is not duplicated | T `dense_matrix::mod_tests::test_from_vec_carries_the_shape` |
| Rank is not a runtime property | T `dense_matrix::mod_tests::test_non_square_in_both_orientations` |
| A sparse matrix answers for a structural zero | T `conversions_tests::test_a_sparse_matrix_with_an_entirely_empty_row` |
| Out-of-bounds access is rejected | T `dense_matrix::mod_tests::test_get_rejects_an_index_outside_the_shape`, `packed_gf2::mod_tests::test_out_of_shape_access_is_rejected` |

## linear-dense-algorithms (15)

| scenario | test |
|---|---|
| One algorithm serves several representations | T `gf2_tests::test_two_word_widths_agree_on_rank`; dense/packed agreement at phase 4 |
| The seam does not degrade to per-element access | 5 — a benchmark, not a test; the 10% budget is measured after implementation |
| A floating-point implementation pivots on magnitude | T `elimination_tests::test_a_near_zero_leading_pivot_does_not_become_the_pivot` |
| An exact implementation pivots on the first non-zero | T `elimination_tests::test_rref_over_an_unordered_field_needs_no_epsilon` |
| A sparse matrix reaches elimination by conversion | P — `CsrMatrix` has no `RowOps` impl; the missing-impl error is the assertion |
| The read side still covers sparse | T `conversions_tests::test_a_sparse_matrix_with_an_entirely_empty_row` |
| A matrix with a zero leading entry | T `elimination_tests::test_a_matrix_with_a_zero_leading_entry_is_still_non_singular` |
| Cayley-Menger volumes are preserved | T `elimination_tests::test_the_tetrahedron_cayley_menger_determinant_is_four`, `..._volume_is_root_two_over_twelve`, `..._right_triangle_...` |
| A genuinely singular matrix still reports zero | T `elimination_tests::test_determinant_of_a_singular_matrix_is_zero` |
| A 3×3 determinant is unchanged | T `elimination_tests::test_determinant_of_a_triangular_matrix_is_the_product_of_its_diagonal` |
| A larger determinant uses elimination | T `elimination_tests::test_a_six_by_six_determinant_uses_elimination_rather_than_expansion` |
| Existing callers compile unchanged | 5 |
| The error type is preserved | 5 — `decomposition_tests` records the variants to compare against |
| A dense matrix uses the same implementation | T `decomposition_tests::*` run against `DenseMatrix` directly |
| Benchmarks are compared across the move | 5 |

## linear-f2-algebra (12)

| scenario | test |
|---|---|
| Rank of a known matrix | T `gf2_tests::test_rank_of_a_known_matrix_over_gf2` |
| Kernel vectors are annihilated | T `gf2_tests::test_kernel_vectors_are_annihilated` |
| Image vectors span the column space | T `gf2_tests::test_the_image_basis_has_rank_elements` |
| The zero matrix | T `gf2_tests::test_the_zero_matrix_has_a_full_kernel_and_an_empty_image` |
| No tolerance parameter exists | P — no 𝔽₂ signature takes one; `rank_gf2` is `(&PackedGf2<W>) -> Result<usize, _>` |
| A matrix where the two ranks differ | T `gf2_tests::test_the_gf2_path_reports_a_smaller_rank_than_the_real_path_where_they_diverge`, `integer_tests::test_the_integer_rank_and_the_mod_two_rank_differ_on_the_same_matrix` |
| Packed elimination outruns the byte-scalar alternative | 5 — a benchmark |
| The advantage grows with size | 5 — a benchmark |
| The duplicated helpers are replaced by one | 5 — in `deep_causality_topology` |
| Existing complexes report unchanged Betti numbers | 5 |
| The choice of field is visible at the call site | T `integer_tests::test_the_integer_rank_and_the_mod_two_rank_differ_on_the_same_matrix` calls each explicitly |
| The gap register is updated | 5 — task 6.10 |

## linear-crate-identity (9)

| scenario | test |
|---|---|
| A matrix algorithm has one home | 5 — the consuming crates still carry theirs until phase 6 |
| N-d operations are untouched | P — the public surface contains no operation over rank ≠ 2 |
| The forbidden direction does not compile | P — `prototype/tensor_impl/` compiles it and gets E0117 |
| No feature reopens the edge | P — `cargo tree` over every feature combination, including dev-deps: zero occurrences of `deep_causality_tensor` |
| The tier graph stays acyclic | P — same measurement |
| A no-std consumer builds | P — `cargo build --no-default-features --features no-std` |
| Lints are inherited | P — `[lints] workspace = true` in the manifest |
| The forbid holds | P — no `unsafe` in `src`; the workspace `forbid` is not overridden |
| Clippy is clean without suppression | P — `cargo clippy -p deep_causality_linear --all-targets`, zero warnings, zero `#[allow(clippy::…)]` |

## Coverage of the corner-case list (task 2.4)

| case | test |
|---|---|
| 0×0, 0×n, n×0 | `dense_matrix::mod_tests::test_zero_by_zero_is_empty_and_square`, `..._zero_by_n_and_n_by_zero_are_distinct_and_both_empty`, `elimination_tests::test_rank_of_an_empty_matrix_is_zero_rather_than_an_error` |
| 1×1 | `dense_matrix::mod_tests::test_one_by_one`, `elimination_tests::test_determinant_of_a_one_by_one_is_its_entry` |
| non-square both ways | `dense_matrix::mod_tests::test_non_square_in_both_orientations`, `elimination_tests::test_rank_of_a_non_square_matrix_in_both_orientations` |
| zero row, zero column | `elimination_tests::test_a_zero_row_and_a_zero_column_do_not_contribute_rank` |
| singular | `elimination_tests::test_determinant_of_a_singular_matrix_is_zero`, `solve_tests::test_a_singular_system_is_rejected_rather_than_answered` |
| non-singular with a zero (0,0) | `elimination_tests::test_a_matrix_with_a_zero_leading_entry_is_still_non_singular`, `solve_tests::test_a_zero_leading_entry_is_handled_by_pivoting` |
| rank-deficient | `elimination_tests::test_rank_of_a_known_rank_deficient_matrix` |
| column count not a multiple of the word width | `packed_gf2::mod_tests::test_a_column_count_that_is_not_a_multiple_of_the_word_width` |
| same matrix at two word widths | `packed_gf2::mod_tests::test_the_same_matrix_at_two_word_widths_reads_identically` |
| {−1,0,1} reduced mod 2 | `packed_gf2::mod_tests::test_the_boundary_operator_alphabet_reduces_mod_two` |
| an entry outside {0,1} offered to the packed constructor | `conversions_tests::test_packing_rejects_an_entry_outside_zero_one_and_names_the_position` |
| an empty CSR row | `conversions_tests::test_a_sparse_matrix_with_an_entirely_empty_row` |
| an out-of-shape index | `dense_matrix::mod_tests::test_get_rejects_an_index_outside_the_shape` |
| near-zero float pivot with a larger one below | `elimination_tests::test_a_near_zero_leading_pivot_does_not_become_the_pivot` |
