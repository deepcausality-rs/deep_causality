/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Num/Float106.lean`.
//!
//! These tests witness the ALGEBRAIC-MODEL laws: the sense in which `Float106`'s arithmetic
//! models the ordered-field laws of the real number it stands for (commutativity of add and mul,
//! left distributivity). The bit-exact double-double error bounds — the limb-level
//! two-sum/two-product behaviour — are out of scope here; the regular `float_double` tests cover
//! those numerics. Comparison uses the same tight epsilon those tests use for real-valued equality.

use deep_causality_num::Float106;

const EPSILON: f64 = 1e-14;

/// Assert two `Float106` values represent the same real number within a tight tolerance.
fn assert_close(a: Float106, b: Float106) {
    assert!(
        (a.hi() - b.hi()).abs() < EPSILON && (a.lo() - b.lo()).abs() < EPSILON,
        "left = ({}, {}), right = ({}, {})",
        a.hi(),
        a.lo(),
        b.hi(),
        b.lo(),
    );
}

/// THEOREM_MAP: num.float106.model.add_comm
#[test]
fn test_float106_add_comm() {
    // Commutativity of addition: a + b = b + a.
    let a = Float106::from_f64(3.25);
    let b = Float106::from_f64(-7.5);
    assert_close(a + b, b + a);
}

/// THEOREM_MAP: num.float106.model.mul_comm
#[test]
fn test_float106_mul_comm() {
    // Commutativity of multiplication: a * b = b * a.
    let a = Float106::from_f64(3.25);
    let b = Float106::from_f64(-7.5);
    assert_close(a * b, b * a);
}

/// THEOREM_MAP: num.float106.model.distrib
#[test]
fn test_float106_distrib() {
    // Left distributivity: a * (b + c) = a * b + a * c.
    let a = Float106::from_f64(3.25);
    let b = Float106::from_f64(-7.5);
    let c = Float106::from_f64(2.0);
    assert_close(a * (b + c), a * b + a * c);
}
