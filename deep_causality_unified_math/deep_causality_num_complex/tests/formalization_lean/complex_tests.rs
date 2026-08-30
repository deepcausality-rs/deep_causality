/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Complex/Complex.lean`.
//!
//! The complex numbers form a field with a norm-multiplicative conjugation. Each test below
//! empirically confirms one Lean theorem on the crate's real `Complex<f64>` type.

use deep_causality_algebra::DivisionAlgebra;
use deep_causality_num_complex::Complex;

// Float comparison tolerance. Products of the sample values stay well within a few ULP of
// their exact results, so 1e-9 leaves ample margin.
const EPSILON: f64 = 1e-9;

/// THEOREM_MAP: complex.field.mul_inv
///
/// For every non-zero `z`, `z * z⁻¹ = 1` (real part 1, imaginary part 0).
#[test]
fn test_complex_field_mul_inv() {
    let samples: [Complex<f64>; 4] = [
        Complex::new(1.0, 2.0),
        Complex::new(-3.0, 4.0),
        Complex::new(5.0, 0.0),  // purely real
        Complex::new(0.0, -2.0), // purely imaginary
    ];
    for z in samples {
        let product = z * z.inverse();
        assert!((product.re() - 1.0).abs() < EPSILON);
        assert!(product.im().abs() < EPSILON);
    }
}

/// THEOREM_MAP: complex.conj.involutive
///
/// Conjugation is an involution: `conjugate(conjugate(z)) = z`.
#[test]
fn test_complex_conj_involutive() {
    let samples: [Complex<f64>; 4] = [
        Complex::new(1.0, 2.0),
        Complex::new(-3.0, 4.0),
        Complex::new(5.0, 0.0),
        Complex::new(0.0, -2.0),
    ];
    for z in samples {
        assert_eq!(z.conjugate().conjugate(), z);
    }
}

/// THEOREM_MAP: complex.conj.mul
///
/// Conjugation distributes over multiplication: `conjugate(z*w) = conjugate(z)*conjugate(w)`.
/// (Complex multiplication is commutative, so no order reversal is observable here.)
#[test]
fn test_complex_conj_mul() {
    let z: Complex<f64> = Complex::new(1.0, 2.0);
    let w: Complex<f64> = Complex::new(-3.0, 4.0);

    let lhs = (z * w).conjugate();
    let rhs = z.conjugate() * w.conjugate();

    assert!((lhs.re() - rhs.re()).abs() < EPSILON);
    assert!((lhs.im() - rhs.im()).abs() < EPSILON);
}

/// THEOREM_MAP: complex.norm_sq.mul
///
/// The squared norm is multiplicative: `norm_sqr(z*w) = norm_sqr(z)*norm_sqr(w)`.
#[test]
fn test_complex_norm_sqr_mul() {
    let z: Complex<f64> = Complex::new(1.0, 2.0);
    let w: Complex<f64> = Complex::new(-3.0, 4.0);

    let lhs = (z * w).norm_sqr();
    let rhs = z.norm_sqr() * w.norm_sqr();

    assert!((lhs - rhs).abs() < EPSILON);
}

/// THEOREM_MAP: complex.norm.mul
///
/// The norm is multiplicative: `|z*w| = |z|*|w|`, with `norm = sqrt(norm_sqr)`.
#[test]
fn test_complex_norm_mul() {
    let z: Complex<f64> = Complex::new(1.0, 2.0);
    let w: Complex<f64> = Complex::new(-3.0, 4.0);

    let lhs = (z * w).norm_sqr().sqrt();
    let rhs = z.norm_sqr().sqrt() * w.norm_sqr().sqrt();

    assert!((lhs - rhs).abs() < EPSILON);
}
