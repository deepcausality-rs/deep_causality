/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The sparse matrix's API and the operator impls that carry it into the tower.

use deep_causality_linear::{CsrMatrix, LinearError, MatrixBuild, MatrixView};
use deep_causality_num::{One, Zero};

fn m(triplets: &[(usize, usize, f64)], r: usize, c: usize) -> CsrMatrix<f64> {
    CsrMatrix::from_triplets(r, c, triplets).unwrap()
}

#[test]
fn test_new_and_default_are_empty() {
    let a: CsrMatrix<f64> = CsrMatrix::new();
    assert_eq!(a.shape(), (0, 0));
    assert!(a.values().is_empty());
    let b: CsrMatrix<f64> = CsrMatrix::default();
    assert_eq!(b.shape(), (0, 0));
}

#[test]
fn test_with_capacity_reserves_without_storing() {
    let a: CsrMatrix<f64> = CsrMatrix::with_capacity(3, 4, 10);
    assert_eq!(a.shape(), (3, 4));
    assert!(a.values().is_empty(), "capacity is not content");
    assert_eq!(a.row_indices().len(), 4, "rows + 1 pointers");
}

#[test]
fn test_from_triplets_drops_explicit_zeros() {
    let a = m(&[(0, 0, 1.0), (1, 1, 0.0)], 2, 2);
    assert_eq!(a.values().len(), 1, "a zero is structural, not stored");
    assert_eq!(a.get_value_at(1, 1), 0.0);
}

#[test]
fn test_into_parts_and_back() {
    let a = m(&[(0, 0, 1.0), (1, 1, 2.0)], 2, 2);
    let (ri, ci, v, shape) = a.into_parts();
    assert_eq!(shape, (2, 2));
    assert_eq!(v, vec![1.0, 2.0]);
    assert_eq!(ci, vec![0, 1]);
    assert_eq!(ri.len(), 3);
}

#[test]
fn test_map_values_visits_only_the_stored_entries() {
    let a = m(&[(0, 0, 1.0), (1, 1, 2.0)], 2, 2);
    let b = a.map_values(|v| v * 10.0);
    assert_eq!(b.get_value_at(0, 0), 10.0);
    assert_eq!(b.get_value_at(1, 1), 20.0);
    assert_eq!(
        b.get_value_at(0, 1),
        0.0,
        "a structural zero is not visited"
    );
}

#[test]
fn test_get_value_at_returns_zero_outside_the_shape_rather_than_panicking() {
    let a = m(&[(0, 0, 1.0)], 2, 2);
    assert_eq!(a.get_value_at(9, 9), 0.0);
}

#[test]
fn test_transpose_moves_entries_and_swaps_the_shape() {
    let a = m(&[(0, 1, 5.0), (2, 0, 7.0)], 3, 2);
    let t = a.transpose();
    assert_eq!(t.shape(), (2, 3));
    assert_eq!(t.get_value_at(1, 0), 5.0);
    assert_eq!(t.get_value_at(0, 2), 7.0);
}

#[test]
fn test_transpose_is_an_involution() {
    let a = m(&[(0, 1, 5.0), (2, 0, 7.0)], 3, 2);
    let back = a.transpose().transpose();
    assert_eq!(back.shape(), a.shape());
    for i in 0..3 {
        for j in 0..2 {
            assert_eq!(back.get_value_at(i, j), a.get_value_at(i, j));
        }
    }
}

#[test]
fn test_vec_mult_against_a_manual_product() {
    // [1 2; 0 3] * [4; 5] = [14; 15]
    let a = m(&[(0, 0, 1.0), (0, 1, 2.0), (1, 1, 3.0)], 2, 2);
    assert_eq!(a.vec_mult(&[4.0, 5.0]).unwrap(), vec![14.0, 15.0]);
}

#[test]
fn test_vec_mult_over_a_row_that_stores_nothing() {
    let a = m(&[(1, 0, 2.0)], 2, 2);
    assert_eq!(a.vec_mult(&[3.0, 4.0]).unwrap(), vec![0.0, 6.0]);
}

#[test]
fn test_mat_mult_against_a_manual_product() {
    // [1 1; 0 1] * [1 0; 1 1] = [2 1; 1 1]
    let a = m(&[(0, 0, 1.0), (0, 1, 1.0), (1, 1, 1.0)], 2, 2);
    let b = m(&[(0, 0, 1.0), (1, 0, 1.0), (1, 1, 1.0)], 2, 2);
    let p = a.mat_mult(&b).unwrap();
    assert_eq!(p.get_value_at(0, 0), 2.0);
    assert_eq!(p.get_value_at(0, 1), 1.0);
    assert_eq!(p.get_value_at(1, 0), 1.0);
    assert_eq!(p.get_value_at(1, 1), 1.0);
}

#[test]
fn test_add_matrix_and_scalar_mult() {
    let a = m(&[(0, 0, 1.0)], 2, 2);
    let b = m(&[(0, 0, 2.0), (1, 1, 3.0)], 2, 2);
    let s = a.add_matrix(&b).unwrap();
    assert_eq!(s.get_value_at(0, 0), 3.0);
    assert_eq!(s.get_value_at(1, 1), 3.0);
    let scaled = b.scalar_mult(2.0);
    assert_eq!(scaled.get_value_at(1, 1), 6.0);
}

#[test]
fn test_add_matrix_rejects_a_shape_mismatch() {
    let a: CsrMatrix<f64> = CsrMatrix::zeros(2, 2);
    let b: CsrMatrix<f64> = CsrMatrix::zeros(2, 3);
    assert!(matches!(
        a.add_matrix(&b),
        Err(LinearError::ShapeMismatch { .. })
    ));
}

#[test]
fn test_the_tower_operators() {
    let a = m(&[(0, 0, 1.0), (1, 1, 2.0)], 2, 2);
    let b = m(&[(0, 0, 3.0)], 2, 2);
    assert_eq!((a.clone() + b.clone()).get_value_at(0, 0), 4.0);
    assert_eq!((a.clone() - b.clone()).get_value_at(0, 0), -2.0);
    assert_eq!((-a.clone()).get_value_at(1, 1), -2.0);
    assert_eq!((a.clone() * b).get_value_at(0, 0), 3.0);
    assert_eq!((a * 5.0).get_value_at(1, 1), 10.0);
}

#[test]
fn test_scalar_multiply_assign() {
    let mut a = m(&[(1, 1, 2.0)], 2, 2);
    a *= 4.0;
    assert_eq!(a.get_value_at(1, 1), 8.0);
}

#[test]
fn test_zero_one_and_their_predicates() {
    let z: CsrMatrix<f64> = CsrMatrix::zero();
    assert!(z.is_zero());
    let o: CsrMatrix<f64> = CsrMatrix::one();
    assert!(o.is_one());
    assert_eq!(o.shape(), (1, 1));
    let i2 = m(&[(0, 0, 1.0), (1, 1, 1.0)], 2, 2);
    assert!(i2.is_one(), "any square identity, not only the 1x1");
    assert!(!m(&[(0, 1, 1.0)], 2, 2).is_one());
    let wide: CsrMatrix<f64> = CsrMatrix::zeros(1, 2);
    assert!(!wide.is_one());
}

#[test]
fn test_the_build_trait_writes_and_clears() {
    let mut a: CsrMatrix<f64> = CsrMatrix::zeros(2, 2);
    a.set(1, 0, 7.0).unwrap();
    assert_eq!(a.get(1, 0).unwrap(), 7.0);
    a.set(1, 0, 9.0).unwrap();
    assert_eq!(
        a.get(1, 0).unwrap(),
        9.0,
        "an existing entry is replaced, not duplicated"
    );
    assert_eq!(a.values().len(), 1);
    a.set(1, 0, 0.0).unwrap();
    assert_eq!(a.values().len(), 0, "writing zero removes the entry");
}

#[test]
fn test_the_read_trait_agrees_with_the_inherent_accessor() {
    let a = m(&[(0, 1, 4.0)], 2, 2);
    for i in 0..2 {
        for j in 0..2 {
            assert_eq!(a.get(i, j).unwrap(), a.get_value_at(i, j));
        }
    }
}
