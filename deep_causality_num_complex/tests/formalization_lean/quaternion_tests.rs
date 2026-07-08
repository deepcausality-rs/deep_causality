/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Complex/Quaternion.lean`.
//!
//! The quaternions form a non-commutative division ring with a norm-multiplicative,
//! order-reversing conjugation. Each test below empirically confirms one Lean theorem on the
//! crate's real `Quaternion<f64>` type.

use deep_causality_algebra::DivisionAlgebra;
use deep_causality_num_complex::Quaternion;

// Float comparison tolerance for the products below.
const EPSILON: f64 = 1e-9;

/// THEOREM_MAP: quaternion.division_ring.mul_inv
///
/// For every non-zero `q`, `q * q⁻¹ = 1` (the real unit quaternion).
#[test]
fn test_quaternion_division_ring_mul_inv() {
    let samples: [Quaternion<f64>; 4] = [
        Quaternion::new(1.0, 2.0, 3.0, 4.0),
        Quaternion::new(-2.0, 0.5, 1.0, -3.0),
        Quaternion::new(5.0, 0.0, 0.0, 0.0), // purely scalar
        Quaternion::new(0.0, 2.0, 0.0, 0.0), // purely vector (2i)
    ];
    for q in samples {
        let product = q * q.inverse();
        assert!((product.w - 1.0).abs() < EPSILON);
        assert!(product.x.abs() < EPSILON);
        assert!(product.y.abs() < EPSILON);
        assert!(product.z.abs() < EPSILON);
    }
}

/// THEOREM_MAP: quaternion.norm_sq.mul
///
/// The squared norm is multiplicative: `norm_sqr(q*p) = norm_sqr(q)*norm_sqr(p)`.
#[test]
fn test_quaternion_norm_sqr_mul() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let p = Quaternion::new(5.0, 6.0, 7.0, 8.0);

    let lhs: f64 = (q * p).norm_sqr();
    let rhs: f64 = q.norm_sqr() * p.norm_sqr();

    assert!((lhs - rhs).abs() < EPSILON);
}

/// THEOREM_MAP: quaternion.conj.mul
///
/// Conjugation is order-reversing (anti-distributive) over multiplication:
/// `conjugate(q*p) = conjugate(p)*conjugate(q)`. Because quaternion multiplication is
/// non-commutative, the order reversal is essential — see `test_quaternion_noncomm`.
#[test]
fn test_quaternion_conj_mul() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let p = Quaternion::new(5.0, 6.0, 7.0, 8.0);

    let lhs: Quaternion<f64> = (q * p).conjugate();
    let rhs: Quaternion<f64> = p.conjugate() * q.conjugate();

    assert!((lhs.w - rhs.w).abs() < EPSILON);
    assert!((lhs.x - rhs.x).abs() < EPSILON);
    assert!((lhs.y - rhs.y).abs() < EPSILON);
    assert!((lhs.z - rhs.z).abs() < EPSILON);
}

/// THEOREM_MAP: quaternion.noncomm
///
/// Quaternion multiplication is non-commutative: the basis units `i` and `j` satisfy
/// `i*j = k` but `j*i = -k`, so `i*j ≠ j*i`.
#[test]
fn test_quaternion_noncomm() {
    let i = Quaternion::new(0.0, 1.0, 0.0, 0.0);
    let j = Quaternion::new(0.0, 0.0, 1.0, 0.0);
    let k = Quaternion::new(0.0, 0.0, 0.0, 1.0);
    let neg_k = Quaternion::new(0.0, 0.0, 0.0, -1.0);

    assert_eq!(i * j, k);
    assert_eq!(j * i, neg_k);
    assert_ne!(i * j, j * i);
}
