/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Algebra/Ring.lean` (Mathlib `Ring`, `CommRing`).
//! `f64` is the representative carrier.

/// THEOREM_MAP: algebra.ring.left_distrib
#[test]
fn test_ring_left_distrib() {
    // a * (b + c) = a * b + a * c
    let (a, b, c) = (2.0f64, 3.0f64, 5.0f64);
    assert_eq!(a * (b + c), a * b + a * c);
}

/// THEOREM_MAP: algebra.ring.right_distrib
#[test]
fn test_ring_right_distrib() {
    // (a + b) * c = a * c + b * c
    let (a, b, c) = (2.0f64, 3.0f64, 5.0f64);
    assert_eq!((a + b) * c, a * c + b * c);
}

/// THEOREM_MAP: algebra.ring.mul_assoc
#[test]
fn test_ring_mul_assoc() {
    // (a * b) * c = a * (b * c)
    let (a, b, c) = (2.0f64, 3.0f64, 5.0f64);
    assert_eq!((a * b) * c, a * (b * c));
}

/// THEOREM_MAP: algebra.commutative_ring.mul_comm
#[test]
fn test_commutative_ring_mul_comm() {
    // a * b = b * a
    let (a, b) = (2.5f64, -4.0f64);
    assert_eq!(a * b, b * a);
}
