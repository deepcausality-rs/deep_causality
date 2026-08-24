/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The operator impls that carry `DenseMatrix` into the tower.

use deep_causality_linear::{DenseMatrix, MatrixBuild, MatrixView};
use deep_causality_num::{One, Zero};

fn m2(d: [f64; 4]) -> DenseMatrix<f64> {
    DenseMatrix::from_vec(d.to_vec(), 2, 2).unwrap()
}

#[test]
fn test_zero_and_is_zero() {
    let z: DenseMatrix<f64> = DenseMatrix::zero();
    assert!(z.is_zero());
    assert_eq!(z.shape(), (0, 0));
    assert!(m2([0.0; 4]).is_zero());
    assert!(!m2([0.0, 0.0, 0.0, 1.0]).is_zero());
}

#[test]
fn test_one_is_the_smallest_identity_and_is_recognised() {
    let o: DenseMatrix<f64> = DenseMatrix::one();
    assert_eq!(o.shape(), (1, 1));
    assert!(o.is_one());
    let i2: DenseMatrix<f64> = DenseMatrix::identity(2);
    assert!(
        i2.is_one(),
        "any square identity must be recognised, not only the 1x1"
    );
    assert!(!m2([1.0, 1.0, 0.0, 1.0]).is_one());
    let wide: DenseMatrix<f64> = DenseMatrix::zeros(1, 2);
    assert!(!wide.is_one(), "a non-square matrix is not the identity");
}

#[test]
fn test_add_and_sub_are_entrywise() {
    let a = m2([1.0, 2.0, 3.0, 4.0]);
    let b = m2([10.0, 20.0, 30.0, 40.0]);
    assert_eq!(
        (a.clone() + b.clone()).as_slice(),
        &[11.0, 22.0, 33.0, 44.0]
    );
    assert_eq!((b - a).as_slice(), &[9.0, 18.0, 27.0, 36.0]);
}

#[test]
fn test_neg_is_entrywise() {
    assert_eq!(
        (-m2([1.0, -2.0, 3.0, -4.0])).as_slice(),
        &[-1.0, 2.0, -3.0, 4.0]
    );
}

#[test]
fn test_mul_is_matrix_multiplication_and_does_not_commute() {
    // [1 1; 0 1] * [1 0; 1 1] = [2 1; 1 1], and the other order gives [1 1; 1 2].
    let a = m2([1.0, 1.0, 0.0, 1.0]);
    let b = m2([1.0, 0.0, 1.0, 1.0]);
    assert_eq!((a.clone() * b.clone()).as_slice(), &[2.0, 1.0, 1.0, 1.0]);
    assert_eq!((b.clone() * a.clone()).as_slice(), &[1.0, 1.0, 1.0, 2.0]);
    assert_ne!(
        (a.clone() * b.clone()).as_slice(),
        (b * a).as_slice(),
        "this is why Commutative<Multiplicative> is absent"
    );
}

#[test]
fn test_the_identity_is_the_multiplicative_unit() {
    let a = m2([1.0, 2.0, 3.0, 4.0]);
    let i: DenseMatrix<f64> = DenseMatrix::identity(2);
    assert_eq!((a.clone() * i.clone()).as_slice(), a.as_slice());
    assert_eq!((i * a.clone()).as_slice(), a.as_slice());
}

#[test]
fn test_a_matrix_with_a_zero_row_is_a_zero_divisor() {
    // Why IntegralDomain is absent: neither factor is zero and the product is.
    let a = m2([1.0, 0.0, 0.0, 0.0]);
    let b = m2([0.0, 0.0, 0.0, 1.0]);
    assert!(!a.is_zero() && !b.is_zero());
    assert!((a * b).is_zero());
}

#[test]
fn test_scalar_multiplication_and_assign() {
    let a = m2([1.0, -2.0, 3.0, -4.0]);
    assert_eq!((a.clone() * 2.0).as_slice(), &[2.0, -4.0, 6.0, -8.0]);
    let mut b = a;
    b *= 3.0;
    assert_eq!(b.as_slice(), &[3.0, -6.0, 9.0, -12.0]);
}

#[test]
fn test_the_ring_laws_on_worked_values() {
    let a = m2([1.0, 2.0, 3.0, 4.0]);
    let b = m2([0.0, 1.0, 1.0, 0.0]);
    let c = m2([2.0, 0.0, 0.0, 3.0]);
    // distributivity: A(B + C) = AB + AC
    let lhs = a.clone() * (b.clone() + c.clone());
    let rhs = a.clone() * b.clone() + a.clone() * c.clone();
    assert_eq!(lhs.as_slice(), rhs.as_slice());
    // associativity of multiplication
    let l = (a.clone() * b.clone()) * c.clone();
    let r = a * (b * c);
    assert_eq!(l.as_slice(), r.as_slice());
}

#[test]
fn test_annihilation_on_worked_values() {
    let a = m2([1.0, 2.0, 3.0, 4.0]);
    let z: DenseMatrix<f64> = DenseMatrix::zeros(2, 2);
    assert!((z.clone() * a.clone()).is_zero());
    assert!((a * z).is_zero());
}

#[test]
#[should_panic(expected = "shape mismatch in add")]
fn test_add_rejects_a_shape_mismatch() {
    let a: DenseMatrix<f64> = DenseMatrix::zeros(2, 2);
    let b: DenseMatrix<f64> = DenseMatrix::zeros(2, 3);
    let _ = a + b;
}

#[test]
#[should_panic(expected = "shape mismatch in sub")]
fn test_sub_rejects_a_shape_mismatch() {
    let a: DenseMatrix<f64> = DenseMatrix::zeros(2, 2);
    let b: DenseMatrix<f64> = DenseMatrix::zeros(3, 2);
    let _ = a - b;
}

#[test]
#[should_panic(expected = "inner dimension mismatch in mul")]
fn test_mul_rejects_an_inner_dimension_mismatch() {
    let a: DenseMatrix<f64> = DenseMatrix::zeros(2, 3);
    let b: DenseMatrix<f64> = DenseMatrix::zeros(4, 2);
    let _ = a * b;
}

#[test]
fn test_the_row_operations_the_elimination_seam_uses() {
    use deep_causality_linear::RowOps;
    let mut a = m2([1.0, 2.0, 3.0, 4.0]);
    a.swap_rows(0, 1).unwrap();
    assert_eq!(a.as_slice(), &[3.0, 4.0, 1.0, 2.0]);
    a.swap_rows(0, 0).unwrap();
    assert_eq!(
        a.as_slice(),
        &[3.0, 4.0, 1.0, 2.0],
        "swapping a row with itself is a no-op"
    );
    a.scale_row(0, &2.0, 0).unwrap();
    assert_eq!(a.as_slice(), &[6.0, 8.0, 1.0, 2.0]);
    a.axpy_rows(1, 0, &-1.0, 0).unwrap();
    assert_eq!(a.as_slice(), &[6.0, 8.0, -5.0, -6.0]);
}

#[test]
fn test_the_row_operations_reject_an_out_of_range_row() {
    use deep_causality_linear::{LinearError, RowOps};
    let mut a = m2([1.0, 2.0, 3.0, 4.0]);
    // Each names the offending row and the shape, so a check against the wrong bound is visible.
    let out_of_range = |r: Result<(), LinearError>| {
        matches!(
            r,
            Err(LinearError::IndexOutOfBounds {
                index: (9, 0),
                shape: (2, 2)
            })
        )
    };
    assert!(out_of_range(a.swap_rows(0, 9)));
    assert!(out_of_range(a.scale_row(9, &2.0, 0)));
    assert!(out_of_range(a.axpy_rows(9, 0, &1.0, 0)));
    // And the matrix is untouched by a rejected operation.
    assert_eq!(a.as_slice(), &[1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_from_col_skips_the_eliminated_prefix() {
    use deep_causality_linear::RowOps;
    // The argument that makes the generic elimination outrun a hand-written loop.
    let mut a = m2([1.0, 2.0, 3.0, 4.0]);
    a.scale_row(0, &10.0, 1).unwrap();
    assert_eq!(
        a.as_slice(),
        &[1.0, 20.0, 3.0, 4.0],
        "column 0 must be untouched"
    );
}
