/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::AsPrimitive;
use deep_causality_num::Quaternion;

#[test]
fn test_as_primitive_f64() {
    let q = Quaternion::new(1.23, 4.56, 7.89, 0.12);
    let val: f64 = q.as_();
    assert!((val - 1.23).abs() < 1e-12);
}

#[test]
fn test_as_primitive_f32() {
    let q = Quaternion::new(1.23f32, 4.56f32, 7.89f32, 0.12f32);
    let val: f32 = q.as_();
    assert!((val - 1.23f32).abs() < 1e-6);
}

#[test]
fn test_as_primitive_i32() {
    let q = Quaternion::new(1.23, 4.56, 7.89, 0.12);
    let val: i32 = q.as_();
    assert_eq!(val, 1);
}

#[test]
fn test_as_primitive_u32() {
    let q = Quaternion::new(1.23, 4.56, 7.89, 0.12);
    let val: u32 = q.as_();
    assert_eq!(val, 1);
}
