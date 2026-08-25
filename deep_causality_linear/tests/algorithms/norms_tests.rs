/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_linear::{
    CsrMatrix, DenseMatrix, DenseVector, MatrixBuild, matrix_norm_frobenius, matrix_norm_inf,
    matrix_norm_l1, vector_norm_inf, vector_norm_l1, vector_norm_sq,
};
use deep_causality_num_complex::Complex;

#[test]
fn test_matrix_norms_on_a_known_matrix() {
    // [1 -2; -3 4]: column sums of moduli 4 and 6; row sums 3 and 7.
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, -2.0, -3.0, 4.0], 2, 2).unwrap();
    assert_eq!(matrix_norm_l1(&m).unwrap(), 6.0);
    assert_eq!(matrix_norm_inf(&m).unwrap(), 7.0);
    // sqrt(1 + 4 + 9 + 16) = sqrt(30)
    assert!((matrix_norm_frobenius(&m).unwrap() - 30.0f64.sqrt()).abs() < 1e-12);
}

#[test]
fn test_the_frobenius_norm_equals_the_two_norm_of_the_flattened_entries() {
    let d = vec![1.0, -2.0, -3.0, 4.0];
    let m = DenseMatrix::from_vec(d.clone(), 2, 2).unwrap();
    let flat: DenseVector<f64> = DenseVector::from_vec(d);
    assert!((matrix_norm_frobenius(&m).unwrap() - flat.norm_l2()).abs() < 1e-12);
}

#[test]
fn test_the_zero_matrix_has_zero_norms_and_no_nan() {
    let m: DenseMatrix<f64> = DenseMatrix::zeros(3, 3);
    for n in [
        matrix_norm_l1(&m).unwrap(),
        matrix_norm_inf(&m).unwrap(),
        matrix_norm_frobenius(&m).unwrap(),
    ] {
        assert_eq!(n, 0.0);
        assert!(!n.is_nan());
    }
}

#[test]
fn test_the_complex_frobenius_norm_uses_the_modulus() {
    let m: DenseMatrix<Complex<f64>> =
        DenseMatrix::from_vec(vec![Complex::new(3.0, 4.0)], 1, 1).unwrap();
    assert!((matrix_norm_frobenius(&m).unwrap() - 5.0).abs() < 1e-12);
}

#[test]
fn test_the_identity_has_unit_norms() {
    let m: DenseMatrix<f64> = DenseMatrix::identity(4);
    assert_eq!(matrix_norm_l1(&m).unwrap(), 1.0);
    assert_eq!(matrix_norm_inf(&m).unwrap(), 1.0);
    assert!(
        (matrix_norm_frobenius(&m).unwrap() - 2.0).abs() < 1e-12,
        "sqrt(4) = 2"
    );
}

#[test]
fn test_the_norms_agree_across_the_dense_and_sparse_representations() {
    // The norms are generic over MatrixView, so the representation is the caller's choice and must
    // not change the answer. Same matrix, same three values, read two different ways.
    //
    // PackedGf2 is absent because its scalar is Gf2, which is not Normed -- an exclusion the type
    // system makes, not one a runtime assertion could observe.
    let dense: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, -2.0, -3.0, 4.0], 2, 2).unwrap();
    let sparse: CsrMatrix<f64> = CsrMatrix::from_triplets(
        2,
        2,
        &[(0, 0, 1.0), (0, 1, -2.0), (1, 0, -3.0), (1, 1, 4.0)],
    )
    .unwrap();

    assert_eq!(matrix_norm_l1(&dense).unwrap(), 6.0);
    assert_eq!(matrix_norm_l1(&sparse).unwrap(), 6.0);
    assert_eq!(matrix_norm_inf(&dense).unwrap(), 7.0);
    assert_eq!(matrix_norm_inf(&sparse).unwrap(), 7.0);
    assert!((matrix_norm_frobenius(&dense).unwrap() - 30.0f64.sqrt()).abs() < 1e-12);
    assert!((matrix_norm_frobenius(&sparse).unwrap() - 30.0f64.sqrt()).abs() < 1e-12);
}

// ---- the unsquared norms do not overflow --------------------------------------------------------

/// `vector_norm_l1(&[1e308])` is `1e308`. It returned infinity while the norms formed each
/// modulus as `modulus_squared().sqrt()`, because `(1e308)²` is not representable.
#[test]
fn test_the_l1_norm_of_a_large_entry_is_finite() {
    assert_eq!(vector_norm_l1(&[1e308_f64]), 1e308);
    assert_eq!(vector_norm_l1(&[3e300_f64, 4e300]), 7e300);
}

#[test]
fn test_the_inf_norm_of_a_large_entry_is_finite() {
    assert_eq!(vector_norm_inf(&[1e308_f64]), 1e308);
    assert_eq!(vector_norm_inf(&[1.0_f64, -1e308, 2.0]), 1e308);
}

/// The other end: a subnormal entry survives, where squaring flushed it to zero.
#[test]
fn test_the_unsquared_norms_keep_a_subnormal_entry() {
    assert_eq!(vector_norm_l1(&[1e-320_f64]), 1e-320);
    assert_eq!(vector_norm_inf(&[1e-320_f64]), 1e-320);
}

/// `norm_sq` is the squared norm and is expected to overflow there — that is what it is for, and
/// the split between the two is the point.
#[test]
fn test_the_squared_norm_still_reports_the_square() {
    assert_eq!(vector_norm_sq(&[3.0_f64, 4.0]), 25.0);
    assert!(vector_norm_sq(&[1e308_f64]).is_infinite());
}

/// A matrix's row and column sums inherit the same property.
#[test]
fn test_the_matrix_one_and_inf_norms_of_large_entries_are_finite() {
    let m = DenseMatrix::from_vec(vec![1e308_f64, 0.0, 0.0, 1e308], 2, 2).unwrap();
    assert_eq!(matrix_norm_l1(&m).unwrap(), 1e308);
    assert_eq!(matrix_norm_inf(&m).unwrap(), 1e308);
}
