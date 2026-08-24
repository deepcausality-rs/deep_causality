/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_linear::{DenseVector, LinearError};
use deep_causality_num_complex::Complex;

#[test]
fn test_round_trip_through_a_slice() {
    let v = DenseVector::from_vec(vec![1.0, 2.0, 3.0]);
    assert_eq!(v.len(), 3);
    assert_eq!(v.as_slice(), &[1.0, 2.0, 3.0]);
    assert!(!v.is_empty());
}

#[test]
fn test_empty_vector() {
    let v: DenseVector<f64> = DenseVector::from_vec(vec![]);
    assert_eq!(v.len(), 0);
    assert!(v.is_empty());
    assert!(v.get(0).is_err());
}

#[test]
fn test_get_rejects_an_index_outside_the_length() {
    let v = DenseVector::from_vec(vec![1.0, 2.0]);
    assert_eq!(v.get(1).unwrap(), 2.0);
    assert!(matches!(
        v.get(2),
        Err(LinearError::IndexOutOfBounds { .. })
    ));
}

#[test]
fn test_dot_agrees_with_the_manual_sum() {
    let a = DenseVector::from_vec(vec![1.0, 2.0, 3.0]);
    let b = DenseVector::from_vec(vec![4.0, 5.0, 6.0]);
    // 1*4 + 2*5 + 3*6
    assert_eq!(a.dot(&b).unwrap(), 32.0);
}

#[test]
fn test_dot_over_the_integers() {
    // The band matters: dot adds and multiplies and does nothing else, so it is available over Z.
    let a = DenseVector::from_vec(vec![1i64, -2, 3]);
    let b = DenseVector::from_vec(vec![4i64, 5, 6]);
    assert_eq!(a.dot(&b).unwrap(), 4 - 10 + 18);
}

#[test]
fn test_dot_over_the_naturals() {
    // And over N, which has no additive inverses but does not need any here.
    let a = DenseVector::from_vec(vec![1u64, 2, 3]);
    let b = DenseVector::from_vec(vec![4u64, 5, 6]);
    assert_eq!(a.dot(&b).unwrap(), 32);
}

#[test]
fn test_dot_rejects_a_length_mismatch_rather_than_truncating() {
    let a = DenseVector::from_vec(vec![1.0, 2.0]);
    let b = DenseVector::from_vec(vec![1.0, 2.0, 3.0]);
    let e = a.dot(&b).unwrap_err();
    assert!(
        matches!(
            e,
            LinearError::LengthMismatch {
                expected: 2,
                found: 3
            }
        ),
        "got {e:?}"
    );
}

#[test]
fn test_outer_product_has_the_expected_shape() {
    use deep_causality_linear::MatrixView;
    let a = DenseVector::from_vec(vec![1.0, 2.0, 3.0]);
    let b = DenseVector::from_vec(vec![4.0, 5.0]);
    let m = a.outer(&b);
    assert_eq!(m.shape(), (3, 2));
    assert_eq!(m.get(0, 0).unwrap(), 4.0);
    assert_eq!(m.get(2, 1).unwrap(), 15.0);
}

#[test]
fn test_add_and_sub() {
    let a = DenseVector::from_vec(vec![1.0, 2.0]);
    let b = DenseVector::from_vec(vec![3.0, 5.0]);
    assert_eq!(a.add(&b).unwrap().as_slice(), &[4.0, 7.0]);
    assert_eq!(b.sub(&a).unwrap().as_slice(), &[2.0, 3.0]);
}

#[test]
fn test_sub_over_the_integers() {
    let a = DenseVector::from_vec(vec![1i64, 2]);
    let b = DenseVector::from_vec(vec![3i64, 5]);
    assert_eq!(a.sub(&b).unwrap().as_slice(), &[-2, -3]);
}

#[test]
fn test_add_and_sub_reject_a_length_mismatch() {
    let a = DenseVector::from_vec(vec![1.0, 2.0]);
    let b = DenseVector::from_vec(vec![1.0]);
    assert!(a.add(&b).is_err());
    assert!(a.sub(&b).is_err());
}

#[test]
fn test_scale() {
    let a = DenseVector::from_vec(vec![1.0, -2.0]);
    assert_eq!(a.scale(3.0).as_slice(), &[3.0, -6.0]);
}

// ---- the Hermitian inner product ---------------------------------------------------------------

#[test]
fn test_over_the_reals_the_two_products_agree() {
    let a = DenseVector::from_vec(vec![1.0, 2.0]);
    let b = DenseVector::from_vec(vec![3.0, 4.0]);
    assert_eq!(a.dot(&b).unwrap(), a.hermitian_inner(&b).unwrap());
}

#[test]
fn test_over_the_complexes_the_inner_product_of_a_vector_with_itself_is_real_and_non_negative() {
    let v: DenseVector<Complex<f64>> =
        DenseVector::from_vec(vec![Complex::new(1.0, 2.0), Complex::new(3.0, -4.0)]);
    let inner = v.hermitian_inner(&v).unwrap();
    assert_eq!(inner.im(), 0.0, "must be real");
    assert!(inner.re() > 0.0, "must be non-negative");
    // |1+2i|^2 + |3-4i|^2 = 5 + 25 = 30
    assert!((inner.re() - 30.0).abs() < 1e-12);
}

#[test]
fn test_the_inner_product_equals_the_square_of_the_two_norm() {
    let v: DenseVector<Complex<f64>> =
        DenseVector::from_vec(vec![Complex::new(1.0, 2.0), Complex::new(3.0, -4.0)]);
    let inner = v.hermitian_inner(&v).unwrap();
    let n = v.norm_l2();
    assert!((inner.re() - n * n).abs() < 1e-12);
}

#[test]
fn test_the_plain_dot_product_over_the_complexes_is_not_an_inner_product() {
    // Why the two are separate: the dot product of a complex vector with itself is neither real
    // nor non-negative, so it induces no norm.
    let v: DenseVector<Complex<f64>> = DenseVector::from_vec(vec![Complex::new(0.0, 1.0)]);
    let d = v.dot(&v).unwrap();
    // i * i = -1
    assert_eq!(d.re(), -1.0);
    let h = v.hermitian_inner(&v).unwrap();
    assert_eq!(h.re(), 1.0);
    assert_ne!(d.re(), h.re());
}

// ---- norms -------------------------------------------------------------------------------------

#[test]
fn test_the_norms_of_three_minus_four() {
    let v: DenseVector<f64> = DenseVector::from_vec(vec![3.0, -4.0]);
    assert_eq!(v.norm_l1(), 7.0);
    assert_eq!(v.norm_l2(), 5.0);
    assert_eq!(v.norm_inf(), 4.0);
    assert_eq!(v.norm_sq(), 25.0);
}

#[test]
fn test_the_complex_two_norm_uses_the_modulus() {
    let v: DenseVector<Complex<f64>> = DenseVector::from_vec(vec![Complex::new(3.0, 4.0)]);
    // |3+4i| = 5
    assert!((v.norm_l2() - 5.0).abs() < 1e-12);
}

#[test]
fn test_the_zero_vector_has_zero_norms_and_no_nan() {
    let v: DenseVector<f64> = DenseVector::from_vec(vec![0.0, 0.0, 0.0]);
    for n in [v.norm_l1(), v.norm_l2(), v.norm_inf(), v.norm_sq()] {
        assert_eq!(n, 0.0);
        assert!(!n.is_nan());
    }
}

#[test]
fn test_the_empty_vector_has_zero_norms_and_no_nan() {
    let v: DenseVector<f64> = DenseVector::from_vec(vec![]);
    for n in [v.norm_l1(), v.norm_l2(), v.norm_inf()] {
        assert_eq!(n, 0.0);
        assert!(!n.is_nan());
    }
}
