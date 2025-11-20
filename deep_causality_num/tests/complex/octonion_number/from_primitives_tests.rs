/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Float, FromPrimitive, Octonion};

#[test]
fn test_octonion_from_isize() {
    let o = Octonion::<f64>::from_isize(123).unwrap();
    assert_eq!(o, Octonion::new(123.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_from_i8() {
    let o = Octonion::<f64>::from_i8(123).unwrap();
    assert_eq!(o, Octonion::new(123.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_from_i16() {
    let o = Octonion::<f64>::from_i16(123).unwrap();
    assert_eq!(o, Octonion::new(123.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_from_i32() {
    let o = Octonion::<f64>::from_i32(123).unwrap();
    assert_eq!(o, Octonion::new(123.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_from_i64() {
    let o = Octonion::<f64>::from_i64(123).unwrap();
    assert_eq!(o, Octonion::new(123.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_from_i128() {
    let o = Octonion::<f64>::from_i128(123).unwrap();
    assert_eq!(o, Octonion::new(123.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_from_usize() {
    let o = Octonion::<f64>::from_usize(123).unwrap();
    assert_eq!(o, Octonion::new(123.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_from_u8() {
    let o = Octonion::<f64>::from_u8(123).unwrap();
    assert_eq!(o, Octonion::new(123.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_from_u16() {
    let o = Octonion::<f64>::from_u16(123).unwrap();
    assert_eq!(o, Octonion::new(123.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_from_u32() {
    let o = Octonion::<f64>::from_u32(123).unwrap();
    assert_eq!(o, Octonion::new(123.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_from_u64() {
    let o = Octonion::<f64>::from_u64(123).unwrap();
    assert_eq!(o, Octonion::new(123.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_from_u128() {
    let o = Octonion::<f64>::from_u128(123).unwrap();
    assert_eq!(o, Octonion::new(123.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_from_f32() {
    let o = Octonion::<f64>::from_f32(123.45).unwrap();
    assert!((o.s - (123.45f32 as f64)).abs() < 1e-6);
    assert_eq!(o.e1, 0.0);
    assert_eq!(o.e7, 0.0);
}

#[test]
fn test_octonion_from_f32_nan() {
    let o = Octonion::<f64>::from_f32(f32::nan());
    assert!(o.unwrap().s.is_nan());
    assert_eq!(o.unwrap().e1, 0.0);
}

#[test]
fn test_octonion_from_f64() {
    let o = Octonion::<f64>::from_f64(123.45).unwrap();
    assert_eq!(o, Octonion::new(123.45, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_from_f64_nan() {
    let o = Octonion::<f64>::from_f64(f64::nan());
    assert!(o.unwrap().s.is_nan());
    assert_eq!(o.unwrap().e1, 0.0);
}
