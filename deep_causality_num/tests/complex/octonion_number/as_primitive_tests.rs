/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{AsPrimitive, Octonion};

#[test]
fn test_octonion_as_primitive_f64() {
    let o = Octonion::new(1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5);
    let val: f64 = o.as_();
    assert_eq!(val, 1.5);
}

#[test]
fn test_octonion_as_primitive_f32() {
    let o = Octonion::new(
        1.5f32, 2.5f32, 3.5f32, 4.5f32, 5.5f32, 6.5f32, 7.5f32, 8.5f32,
    );
    let val: f32 = o.as_();
    assert_eq!(val, 1.5f32);
}

#[test]
fn test_octonion_as_primitive_i32() {
    let o = Octonion::new(1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5);
    let val: i32 = o.as_();
    assert_eq!(val, 1);
}

#[test]
fn test_octonion_as_primitive_u32() {
    let o = Octonion::new(1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5);
    let val: u32 = o.as_();
    assert_eq!(val, 1);
}
