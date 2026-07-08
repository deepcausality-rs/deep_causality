/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Num/Identity.lean`.
//!
//! Pins the two-sided identity laws of the additive and multiplicative monoids to the crate's
//! real `Zero` and `One` marker traits. Lean proves the laws for all elements; these tests check
//! them empirically on a concrete numeric type at representative inputs.

use deep_causality_num::{One, Zero};

/// THEOREM_MAP: num.one.identity
#[test]
fn test_one_identity() {
    // Two-sided multiplicative identity: 1 * a = a and a * 1 = a.
    let one: i64 = One::one();
    for a in [-7i64, 0, 1, 42, i64::MAX] {
        assert_eq!(one * a, a);
        assert_eq!(a * one, a);
    }
}

/// THEOREM_MAP: num.zero.identity
#[test]
fn test_zero_identity() {
    // Two-sided additive identity: 0 + a = a and a + 0 = a.
    let zero: i64 = Zero::zero();
    for a in [-7i64, 0, 1, 42, i64::MAX] {
        assert_eq!(zero + a, a);
        assert_eq!(a + zero, a);
    }
}
