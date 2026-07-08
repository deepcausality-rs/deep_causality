/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Num/Integer.lean`.
//!
//! Pins the commutative-ring and Euclidean-division laws to the crate's real `Integer` trait.
//! Lean proves the laws for all integers; these tests check them empirically on `i64` at
//! representative inputs, including a negative dividend for the Euclidean law.

use deep_causality_num::Integer;

/// THEOREM_MAP: num.integer.mul_comm
#[test]
fn test_integer_mul_comm() {
    // Commutativity of integer multiplication: a * b = b * a.
    for (a, b) in [(3i64, 7i64), (-4, 9), (0, 15), (-6, -8)] {
        assert_eq!(a * b, b * a);
    }
}

/// THEOREM_MAP: num.integer.distrib
#[test]
fn test_integer_distrib() {
    // Left distributivity: a * (b + c) = a * b + a * c.
    for (a, b, c) in [(3i64, 7i64, 2i64), (-4, 9, -5), (0, 15, 11), (-6, -8, 4)] {
        assert_eq!(a * (b + c), a * b + a * c);
    }
}

/// THEOREM_MAP: num.integer.euclidean
#[test]
fn test_integer_euclidean() {
    // Euclidean reconstruction: b * a.div_euclid(b) + a.rem_euclid(b) = a.
    // Rust's `rem_euclid` is always non-negative, so the identity holds for a negative
    // dividend as well, which makes the check meaningful beyond the positive case.
    let check = |a: i64, b: i64| {
        let q = Integer::div_euclid(a, b);
        let r = Integer::rem_euclid(a, b);
        assert_eq!(b * q + r, a);
    };

    check(17, 5);
    check(-17, 5); // negative dividend
    check(17, -5); // negative divisor
    check(-17, -5);
    check(0, 7);
}
