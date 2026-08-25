/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The dense kernels, reached through the decomposition entry points that wrap them.
//!
//! These drive the guards the kernels carry for inputs the ordinary formulas cannot be written for:
//! a reflector whose pivot entry is zero, and a matrix small enough that a product of two column
//! norms underflows.

use deep_causality_linear::{DenseMatrix, MatrixView, qr, singular_values, svd};

#[test]
fn test_qr_of_a_matrix_whose_pivot_entry_is_zero() {
    // The Householder phase is normally taken from the pivot entry, `r[j][j] / |r[j][j]|`, which is
    // undefined when that entry is zero while the column below it is not. The permutation matrix
    // [[0,1],[1,0]] is exactly that case at j = 0, and it is orthogonal, so Q and R are determined:
    // R must come out upper triangular and QR must reproduce A.
    let a = DenseMatrix::from_vec(vec![0.0, 1.0, 1.0, 0.0], 2, 2).unwrap();
    let (q, r) = qr(&a).unwrap();
    assert_eq!(q.shape(), (2, 2));
    assert_eq!(r.shape(), (2, 2));

    assert_eq!(
        r.get(1, 0).unwrap(),
        0.0,
        "R is upper triangular, exactly rather than to rounding"
    );

    // QᵀQ = I: the reflector left orthonormal columns rather than a NaN.
    for i in 0..2 {
        for j in 0..2 {
            let mut acc = 0.0_f64;
            for k in 0..2 {
                acc += q.get(k, i).unwrap() * q.get(k, j).unwrap();
            }
            let want = if i == j { 1.0 } else { 0.0 };
            assert!((acc - want).abs() < 1e-12, "QᵀQ at ({i}, {j}) was {acc}");
        }
    }

    // QR = A.
    for i in 0..2 {
        for j in 0..2 {
            let mut acc = 0.0_f64;
            for k in 0..2 {
                acc += q.get(i, k).unwrap() * r.get(k, j).unwrap();
            }
            assert!(
                (acc - a.get(i, j).unwrap()).abs() < 1e-12,
                "QR at ({i}, {j}) was {acc}"
            );
        }
    }
}

#[test]
fn test_the_singular_values_of_a_matrix_whose_squared_column_norms_multiply_below_the_exponent_range()
 {
    // The Jacobi sweep measures the off-diagonal relative to `sqrt(αβ)`, where α and β are squared
    // column norms. At entries of 2⁻²⁸³ each column norm squared is 2⁻⁵⁶⁶, and their product 2⁻¹¹³²
    // is below the smallest subnormal double, so that scale collapses to zero and the relative test
    // cannot be formed. The columns of a diagonal matrix are already orthogonal, so the answer is
    // still the moduli of the diagonal, exactly -- 2⁻²⁸³ is a power of two and squares and roots
    // back without rounding.
    let tiny = 2.0_f64.powi(-283);
    assert!(tiny > 0.0, "the entry itself is representable");
    assert_eq!(tiny * tiny * (tiny * tiny), 0.0, "its fourth power is not");

    let a = DenseMatrix::from_vec(vec![tiny, 0.0, 0.0, tiny], 2, 2).unwrap();
    let s = singular_values(&a).unwrap();
    assert_eq!(s.len(), 2);
    assert_eq!(s.get(0).unwrap(), tiny);
    assert_eq!(s.get(1).unwrap(), tiny);

    // And the factors still reconstruct: U = V = I for a positive diagonal matrix.
    let (u, _, vt) = svd(&a).unwrap();
    assert_eq!(u.shape(), (2, 2));
    assert_eq!(vt.shape(), (2, 2));
    for i in 0..2 {
        for j in 0..2 {
            let mut acc = 0.0;
            for k in 0..2 {
                acc += u.get(i, k).unwrap() * s.get(k).unwrap() * vt.get(k, j).unwrap();
            }
            assert_eq!(acc, a.get(i, j).unwrap(), "reconstruction at ({i}, {j})");
        }
    }
}
