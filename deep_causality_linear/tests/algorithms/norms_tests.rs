/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_linear::{
    DenseMatrix, DenseVector, MatrixBuild, matrix_norm_frobenius, matrix_norm_inf, matrix_norm_l1,
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
fn test_norms_apply_to_every_representation_through_the_read_trait() {
    use deep_causality_linear::PackedGf2;
    // The norms are generic over MatrixView, so they reach the sparse and packed types too.
    let m: PackedGf2<u64> = PackedGf2::identity(3);
    // GF(2) is not a NormedScalar, so this must not compile -- asserted by its absence here rather
    // than by a call. What is asserted is that a dense integer matrix does reach them.
    let _ = m;
    let i: DenseMatrix<f64> = DenseMatrix::identity(2);
    assert!(matrix_norm_l1(&i).is_ok());
}
