/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! One instantiation per row of `openspec/notes/archive/linear/BOUND-LEDGER.md`.
//!
//! A bound loosened off `Field` or `RealField` that nothing exercises is indistinguishable from an
//! untested one, and the loosening is the moment a body can quietly stop being correct for the types
//! it now takes. Each test here calls the loosened operation at the number set it newly admits.

use deep_causality_linear::{
    DenseMatrix, DenseVector, MatrixBuild, MatrixView, determinant_exact, matrix_norm_frobenius,
    rank_exact,
};
use deep_causality_num_complex::Complex;

// ---- loosened to CommutativeSemiring: admits ℕ -------------------------------------------------

#[test]
fn test_dot_over_the_naturals() {
    let a = DenseVector::from_vec(vec![1u64, 2, 3]);
    let b = DenseVector::from_vec(vec![4u64, 5, 6]);
    assert_eq!(a.dot(&b).unwrap(), 32);
}

#[test]
fn test_a_natural_matrix_can_be_built_and_read() {
    let m = DenseMatrix::from_vec(vec![1u64, 2, 3, 4], 2, 2).unwrap();
    assert_eq!(m.get(1, 1).unwrap(), 4);
}

// ---- loosened to CommutativeRing: admits ℤ ------------------------------------------------------

#[test]
fn test_vector_add_sub_and_scale_over_the_integers() {
    let a = DenseVector::from_vec(vec![1i64, 2]);
    let b = DenseVector::from_vec(vec![10i64, 20]);
    assert_eq!(b.sub(&a).unwrap().as_slice(), &[9, 18]);
    assert_eq!(a.add(&b).unwrap().as_slice(), &[11, 22]);
    assert_eq!(a.scale(-3).as_slice(), &[-3, -6]);
}

#[test]
fn test_outer_product_over_the_integers() {
    let a = DenseVector::from_vec(vec![1i64, 2]);
    let b = DenseVector::from_vec(vec![3i64, 4]);
    let m = a.outer(&b);
    assert_eq!(m.shape(), (2, 2));
    assert_eq!(m.get(1, 1).unwrap(), 8);
}

// ---- loosened to EuclideanDomain: admits ℤ exactly ----------------------------------------------

#[test]
fn test_the_integer_determinant_is_reachable_over_i64() {
    let m = DenseMatrix::from_vec(vec![3i64, 1, 2, 4], 2, 2).unwrap();
    assert_eq!(determinant_exact(&m).unwrap(), 10);
}

#[test]
fn test_the_integer_rank_is_reachable_over_i64() {
    let m: DenseMatrix<i64> = DenseMatrix::identity(3);
    assert_eq!(rank_exact(&m).unwrap(), 3);
}

// ---- loosened to NormedScalar: admits ℂ ---------------------------------------------------------

#[test]
fn test_the_frobenius_norm_over_the_complexes() {
    let m: DenseMatrix<Complex<f64>> =
        DenseMatrix::from_vec(vec![Complex::new(3.0, 4.0)], 1, 1).unwrap();
    assert!((matrix_norm_frobenius(&m).unwrap() - 5.0).abs() < 1e-12);
}

// ---- loosened to ConjugateScalar: admits ℂ ------------------------------------------------------

#[test]
fn test_the_hermitian_inner_product_over_the_complexes() {
    let v: DenseVector<Complex<f64>> = DenseVector::from_vec(vec![Complex::new(0.0, 1.0)]);
    let inner = v.hermitian_inner(&v).unwrap();
    assert_eq!(inner.re(), 1.0);
    assert_eq!(inner.im(), 0.0);
}

// ---- deliberately not loosened ------------------------------------------------------------------

#[test]
fn test_the_pivoted_determinant_runs_over_the_complex_numbers() {
    // solve, Lu, inverse and the pivoted determinant pivot by magnitude, so they need a modulus
    // landing in an ordered real. That admits R, C and Float106 and no more, which is why an exact
    // solve over Q is a later entry point rather than a widening of this one.
    //
    // What is checked here is the running, not the admission: C reaching NormedScalar is settled by
    // the call compiling, so the assertion has to be the answer the complex path computes.
    use deep_causality_linear::determinant;
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], 2, 2).unwrap();
    assert_eq!(determinant(&m).unwrap(), -2.0);

    // [1+i, 2; 3, 4-i]: (1+i)(4-i) - 6 = (5 + 3i) - 6 = -1 + 3i.
    let c: DenseMatrix<Complex<f64>> = DenseMatrix::from_vec(
        vec![
            Complex::new(1.0, 1.0),
            Complex::new(2.0, 0.0),
            Complex::new(3.0, 0.0),
            Complex::new(4.0, -1.0),
        ],
        2,
        2,
    )
    .unwrap();
    let d = determinant(&c).unwrap();
    assert!((d.re - (-1.0)).abs() < 1e-12, "re was {}", d.re);
    assert!((d.im - 3.0).abs() < 1e-12, "im was {}", d.im);
}
