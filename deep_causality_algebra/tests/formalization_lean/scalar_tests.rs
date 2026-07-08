/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Algebra/Scalar.lean` (Mathlib `StarRing`,
//! `NormedField`). `f64` is the representative carrier: conjugation is the identity and the
//! modulus is `x²`, reached through `ConjugateScalar` and `Normed`.

use deep_causality_algebra::{ConjugateScalar, Normed};

/// THEOREM_MAP: algebra.conjugate.star_star
#[test]
fn test_conjugate_star_star() {
    // conjugate (conjugate a) = a
    for a in [2.0f64, -3.5, 7.25] {
        let star = <f64 as ConjugateScalar>::conjugate(&a);
        assert_eq!(<f64 as ConjugateScalar>::conjugate(&star), a);
    }
}

/// THEOREM_MAP: algebra.conjugate.star_mul
#[test]
fn test_conjugate_star_mul() {
    // conjugate (a * b) = conjugate b * conjugate a (on reals both sides equal a * b)
    let (a, b) = (2.0f64, -3.5f64);
    let lhs = <f64 as ConjugateScalar>::conjugate(&(a * b));
    let rhs = <f64 as ConjugateScalar>::conjugate(&b) * <f64 as ConjugateScalar>::conjugate(&a);
    assert_eq!(lhs, rhs);
}

/// THEOREM_MAP: algebra.conjugate.star_add
#[test]
fn test_conjugate_star_add() {
    // conjugate (a + b) = conjugate a + conjugate b
    let (a, b) = (2.0f64, -3.5f64);
    let lhs = <f64 as ConjugateScalar>::conjugate(&(a + b));
    let rhs = <f64 as ConjugateScalar>::conjugate(&a) + <f64 as ConjugateScalar>::conjugate(&b);
    assert_eq!(lhs, rhs);
}

/// THEOREM_MAP: algebra.normed.norm_mul
#[test]
fn test_normed_norm_mul() {
    // The squared modulus is multiplicative: |a·b|² = |a|²·|b|² (for reals).
    let (a, b) = (2.0f64, -3.5f64);
    assert_eq!(
        <f64 as Normed>::modulus_squared(&(a * b)),
        <f64 as Normed>::modulus_squared(&a) * <f64 as Normed>::modulus_squared(&b)
    );
}

/// THEOREM_MAP: algebra.normed.norm_nonneg
#[test]
fn test_normed_norm_nonneg() {
    // 0 ≤ |a|²
    for a in [2.0f64, -3.5, 0.0, 7.25] {
        assert!(<f64 as Normed>::modulus_squared(&a) >= 0.0);
    }
}
