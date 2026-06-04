/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Dual, Float106, FromPrimitive};

#[test]
fn test_from_f64_is_a_constant_dual() {
    let d: Dual<f64> = Dual::from_f64(3.5).unwrap();
    assert_eq!(d.value(), 3.5);
    assert_eq!(d.derivative(), 0.0); // a constant has zero derivative
}

#[test]
fn test_from_f32_lifts_at_f32_precision() {
    // The point of FromPrimitive over From<f64>: f32 works.
    let d: Dual<f32> = Dual::from_f64(1.25).unwrap();
    assert_eq!(d.value(), 1.25_f32);
    assert_eq!(d.derivative(), 0.0_f32);
}

#[test]
fn test_from_primitive_nests_through_dual_of_dual() {
    // Dual<Dual<f32>> is FromPrimitive too — the constant lift survives nesting.
    let d: Dual<Dual<f32>> = Dual::from_f64(2.0).unwrap();
    assert_eq!(d.value().value(), 2.0_f32);
    assert_eq!(d.value().derivative(), 0.0_f32);
    assert_eq!(d.derivative().value(), 0.0_f32);
    assert_eq!(d.derivative().derivative(), 0.0_f32);
}

#[test]
fn test_from_primitive_high_precision() {
    // 0.5 is exactly representable, so the lifted value is exact at Float106 precision.
    let d: Dual<Float106> = Dual::from_f64(0.5).unwrap();
    assert_eq!(d.value(), Float106::from(0.5));
    assert_eq!(d.derivative(), Float106::from(0.0));
}

#[test]
fn test_from_integer_primitives() {
    let a: Dual<f64> = Dual::from_i32(7).unwrap();
    assert_eq!(a.value(), 7.0);
    let b: Dual<f64> = Dual::from_u64(9).unwrap();
    assert_eq!(b.value(), 9.0);
    let c: Dual<f64> = Dual::from_usize(3).unwrap();
    assert_eq!(c.value(), 3.0);
}
