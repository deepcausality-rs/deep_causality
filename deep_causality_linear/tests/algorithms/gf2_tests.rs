/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Exact mod-2 elimination: the closure condition for qcl-gaps.md G-01.

use deep_causality_linear::utils_tests::fixtures_matrix::*;
use deep_causality_linear::{
    MatrixBuild, MatrixView, PackedGf2, image_basis_gf2, kernel_basis_gf2, rank_gf2,
};
use deep_causality_num::Gf2;

#[test]
fn test_rank_of_a_known_matrix_over_gf2() {
    let (d, r, c) = ranks_disagree_3x3();
    let m: PackedGf2<u64> = PackedGf2::from_i64_mod2(&d, r, c).unwrap();
    let over_f2 = rank_gf2(&m).unwrap();
    assert_eq!(over_f2, RANKS_DISAGREE_GF2_RANK);
    assert!(
        over_f2 < RANKS_DISAGREE_RATIONAL_RANK,
        "the F2 rank must be the smaller one"
    );
}

#[test]
fn test_rank_of_the_zero_matrix_is_zero() {
    let m: PackedGf2<u64> = PackedGf2::zeros(3, 3);
    assert_eq!(rank_gf2(&m).unwrap(), 0);
}

#[test]
fn test_rank_of_the_identity_is_its_order() {
    let m: PackedGf2<u8> = PackedGf2::identity(5);
    assert_eq!(rank_gf2(&m).unwrap(), 5);
}

#[test]
fn test_kernel_vectors_are_annihilated() {
    let (d, r, c) = ranks_disagree_3x3();
    let m: PackedGf2<u64> = PackedGf2::from_i64_mod2(&d, r, c).unwrap();
    let kernel = kernel_basis_gf2(&m).unwrap();
    // M * v = 0 over F2 for every column v of the kernel basis.
    for k in 0..kernel.cols() {
        for i in 0..m.rows() {
            let mut acc = Gf2::ZERO;
            for j in 0..m.cols() {
                acc += m.get(i, j).unwrap() * kernel.get(j, k).unwrap();
            }
            assert_eq!(
                acc,
                Gf2::ZERO,
                "kernel vector {k} not annihilated at row {i}"
            );
        }
    }
}

#[test]
fn test_the_kernel_basis_has_cols_minus_rank_elements() {
    let (d, r, c) = ranks_disagree_3x3();
    let m: PackedGf2<u64> = PackedGf2::from_i64_mod2(&d, r, c).unwrap();
    let rank = rank_gf2(&m).unwrap();
    let kernel = kernel_basis_gf2(&m).unwrap();
    assert_eq!(kernel.cols(), m.cols() - rank);
}

#[test]
fn test_the_image_basis_has_rank_elements() {
    let (d, r, c) = ranks_disagree_3x3();
    let m: PackedGf2<u64> = PackedGf2::from_i64_mod2(&d, r, c).unwrap();
    let rank = rank_gf2(&m).unwrap();
    assert_eq!(image_basis_gf2(&m).unwrap().cols(), rank);
}

#[test]
fn test_the_zero_matrix_has_a_full_kernel_and_an_empty_image() {
    let m: PackedGf2<u64> = PackedGf2::zeros(3, 4);
    assert_eq!(rank_gf2(&m).unwrap(), 0);
    assert_eq!(kernel_basis_gf2(&m).unwrap().cols(), 4);
    assert_eq!(image_basis_gf2(&m).unwrap().cols(), 0);
}

#[test]
fn test_two_word_widths_agree_on_rank() {
    let (d, r, c) = ranks_disagree_3x3();
    let narrow: PackedGf2<u8> = PackedGf2::from_i64_mod2(&d, r, c).unwrap();
    let wide: PackedGf2<u64> = PackedGf2::from_i64_mod2(&d, r, c).unwrap();
    assert_eq!(rank_gf2(&narrow).unwrap(), rank_gf2(&wide).unwrap());
}

#[test]
fn test_a_column_count_past_a_word_boundary_does_not_change_the_rank() {
    // At u8 the boundary is column 8, so a 10-column matrix crosses it inside one row.
    let mut narrow: PackedGf2<u8> = PackedGf2::zeros(2, 10);
    let mut wide: PackedGf2<u64> = PackedGf2::zeros(2, 10);
    for (i, j) in [(0usize, 0usize), (0, 9), (1, 5)] {
        narrow.set(i, j, Gf2::ONE).unwrap();
        wide.set(i, j, Gf2::ONE).unwrap();
    }
    assert_eq!(rank_gf2(&narrow).unwrap(), 2);
    assert_eq!(rank_gf2(&narrow).unwrap(), rank_gf2(&wide).unwrap());
}

#[test]
fn test_the_gf2_path_reports_a_smaller_rank_than_the_real_path_where_they_diverge() {
    let (d, r, c) = ranks_disagree_3x3();
    let m: PackedGf2<u64> = PackedGf2::from_i64_mod2(&d, r, c).unwrap();
    assert_eq!(rank_gf2(&m).unwrap(), RANKS_DISAGREE_GF2_RANK);
}
