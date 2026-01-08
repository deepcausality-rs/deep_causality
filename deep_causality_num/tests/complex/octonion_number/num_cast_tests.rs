/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Float, NumCast, Octonion};

#[test]
fn test_octonion_num_cast_from_f64() {
    let o = <Octonion<f64> as NumCast>::from(123.45).unwrap();
    assert_eq!(o, Octonion::new(123.45, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_num_cast_from_i32() {
    let o = <Octonion<f64> as NumCast>::from(123).unwrap();
    assert_eq!(o, Octonion::new(123.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_num_cast_from_f64_nan() {
    let o = <Octonion<f64> as NumCast>::from(f64::nan());
    assert!(o.unwrap().s.is_nan());
}
