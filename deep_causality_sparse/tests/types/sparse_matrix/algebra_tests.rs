/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{One, Zero};
use deep_causality_sparse::CsrMatrix;

#[test]
fn test_zero_matrix() {
    let z: CsrMatrix<f64> = CsrMatrix::zero(3, 3);
    assert_eq!(z.shape(), (3, 3));
    assert_eq!(z.values().len(), 0); // No non-zero elements
}

#[test]
fn test_additive_identity() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    let z: CsrMatrix<f64> = CsrMatrix::zero(2, 2); // Added type annotation
    let b = &a + &z;
    assert_eq!(a, b); // A + 0 = A
}

#[test]
fn test_additive_inverse() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    let neg_a = a.neg();
    let z = &a + &neg_a;
    // For sparse matrices, A + (-A) should result in an empty matrix (no non-zero elements)
    // with the same shape.
    assert_eq!(z.values().len(), 0);
    assert_eq!(z.shape(), a.shape());
}

#[test]
fn test_commutativity() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    let b = CsrMatrix::from_triplets(2, 2, &[(1, 1, 2.0)]).unwrap();
    let ab = &a + &b;
    let ba = &b + &a;
    assert_eq!(ab, ba); // A + B = B + A
}

#[test]
fn test_associativity() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    let b = CsrMatrix::from_triplets(2, 2, &[(1, 1, 2.0)]).unwrap();
    let c = CsrMatrix::from_triplets(2, 2, &[(0, 1, 3.0)]).unwrap();
    let ab_c = (&a + &b) + &c;
    let a_bc = &a + (&b + &c);
    assert_eq!(ab_c, a_bc); // (A + B) + C = A + (B + C)
}

#[test]
fn test_scalar_multiplication() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    let b = a.scale(3.0);
    assert_eq!(b.get_value_at(0, 0), 3.0);
    assert_eq!(b.get_value_at(1, 1), 6.0);
}

#[test]
fn test_matrix_multiplication_identity() {
    let i: CsrMatrix<f64> = CsrMatrix::one(3); // Added type annotation
    let a = CsrMatrix::from_triplets(3, 3, &[(0, 0, 1.0), (1, 1, 2.0), (2, 2, 3.0)]).unwrap();
    let b = &i * &a;
    assert_eq!(a, b); // I * A = A
}

#[test]
#[should_panic(expected = "shape mismatch")]
fn test_add_shape_mismatch() {
    let a: CsrMatrix<f64> = CsrMatrix::zero(2, 2); // Added type annotation
    let b: CsrMatrix<f64> = CsrMatrix::zero(3, 3); // Added type annotation
    let _c = a + b; // Should panic
}

#[test]
fn test_scalar_zero_trait() {
    let scalar_zero: CsrMatrix<f64> = Zero::zero();
    assert_eq!(scalar_zero.shape(), (0, 0));
    assert!(scalar_zero.is_zero());

    let m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    assert!(!m.is_zero());
}

#[test]
fn test_scalar_one_trait() {
    let scalar_one: CsrMatrix<f64> = One::one();
    assert_eq!(scalar_one.shape(), (1, 1));
    assert!(scalar_one.is_one());

    let m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    assert!(!m.is_one());
}

#[test]
fn test_from_triplets_with_zero() {
    // Use 1.0 as "zero" - elements with value 1.0 should be excluded
    let triplets = vec![
        (0, 0, 1.0), // Should be excluded
        (0, 1, 2.0), // Should be included
        (1, 0, 3.0), // Should be included
        (1, 1, 1.0), // Should be excluded
    ];

    let matrix = CsrMatrix::from_triplets_with_zero(2, 2, &triplets, 1.0).unwrap();

    assert_eq!(matrix.shape(), (2, 2));
    assert_eq!(matrix.values(), &vec![2.0, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![1, 0]);
    // Row indices:
    // Row 0: 1 element (2.0 at col 1) -> starts at 0, ends at 1
    // Row 1: 1 element (3.0 at col 0) -> starts at 1, ends at 2
    assert_eq!(matrix.row_indices(), &vec![0, 1, 2]);
}

#[test]
fn test_add_with_zero() {
    // Use 0.0 as zero for standard addition check
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    let b = CsrMatrix::from_triplets(2, 2, &[(0, 0, 2.0), (1, 1, 3.0)]).unwrap();

    // 1.0 + 2.0 = 3.0
    // 2.0 + 3.0 = 5.0
    let c = a.add_with_zero(&b, 0.0).unwrap();

    assert_eq!(c.values(), &vec![3.0, 5.0]);

    // Test cancellation with explicit zero
    // A = [1.0, 0.0]
    // B = [-1.0, 0.0]
    // Sum = [0.0, 0.0] -> Empty if zero is 0.0
    let a2 = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    let b2 = CsrMatrix::from_triplets(2, 2, &[(0, 0, -1.0)]).unwrap();
    let c2 = a2.add_with_zero(&b2, 0.0).unwrap();
    assert!(c2.values().is_empty());

    // Test with non-standard zero
    // Treat 3.0 as zero.
    // A = [1.0, 2.0]
    // B = [2.0, 1.0]
    // Sum = [3.0, 3.0] -> Both should be filtered out if zero is 3.0
    let a3 = CsrMatrix::from_triplets_with_zero(1, 2, &[(0, 0, 1.0), (0, 1, 2.0)], 0.0).unwrap();
    let b3 = CsrMatrix::from_triplets_with_zero(1, 2, &[(0, 0, 2.0), (0, 1, 1.0)], 0.0).unwrap();
    let c3 = a3.add_with_zero(&b3, 3.0).unwrap();
    assert!(c3.values().is_empty());
}

#[test]
fn test_module_trait() {
    let matrix = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();

    // Scale by 2.0
    let scaled = matrix.scale(2.0);

    assert_eq!(scaled.values(), &vec![2.0, 4.0]);

    // Verify it's a new matrix
    assert_ne!(matrix.values(), scaled.values());
}
