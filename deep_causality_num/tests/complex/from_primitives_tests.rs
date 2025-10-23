/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Complex, Float, FromPrimitive};

#[test]
fn test_from_isize_ok() {
    let c = Complex::<f64>::from_isize(10).unwrap();
    assert_eq!(c.re, 10.0f64);
    assert_eq!(c.im, 0.0f64);
}

#[test]
fn test_from_isize_f32_precision_loss() {
    // isize::MAX is too large for f32 to represent precisely, but it can represent its magnitude.
    // f32::from_isize will return Some with a rounded value.
    let c = Complex::<f32>::from_isize(isize::MAX).unwrap();
    assert_eq!(c.re, isize::MAX as f32);
    assert_eq!(c.im, 0.0f32);
}

#[test]
fn test_from_i8_ok() {
    let c = Complex::<f64>::from_i8(10).unwrap();
    assert_eq!(c.re, 10.0f64);
    assert_eq!(c.im, 0.0f64);
}

#[test]
fn test_from_i16_ok() {
    let c = Complex::<f64>::from_i16(1000).unwrap();
    assert_eq!(c.re, 1000.0f64);
    assert_eq!(c.im, 0.0f64);
}

#[test]
fn test_from_i32_ok() {
    let c = Complex::<f64>::from_i32(100000).unwrap();
    assert_eq!(c.re, 100000.0f64);
    assert_eq!(c.im, 0.0f64);
}

#[test]
fn test_from_i64_ok() {
    let c = Complex::<f64>::from_i64(10000000000).unwrap();
    assert_eq!(c.re, 10000000000.0f64);
    assert_eq!(c.im, 0.0f64);
}

#[test]
fn test_from_i128_ok() {
    let c = Complex::<f64>::from_i128(100000000000000000).unwrap();
    assert_eq!(c.re, 100000000000000000.0f64);
    assert_eq!(c.im, 0.0f64);
}

#[test]
fn test_from_usize_ok() {
    let c = Complex::<f64>::from_usize(10).unwrap();
    assert_eq!(c.re, 10.0f64);
    assert_eq!(c.im, 0.0f64);
}

#[test]
fn test_from_u8_ok() {
    let c = Complex::<f64>::from_u8(10).unwrap();
    assert_eq!(c.re, 10.0f64);
    assert_eq!(c.im, 0.0f64);
}

#[test]
fn test_from_u16_ok() {
    let c = Complex::<f64>::from_u16(1000).unwrap();
    assert_eq!(c.re, 1000.0f64);
    assert_eq!(c.im, 0.0f64);
}

#[test]
fn test_from_u32_ok() {
    let c = Complex::<f64>::from_u32(100000).unwrap();
    assert_eq!(c.re, 100000.0f64);
    assert_eq!(c.im, 0.0f64);
}

#[test]
fn test_from_u64_ok() {
    let c = Complex::<f64>::from_u64(10000000000).unwrap();
    assert_eq!(c.re, 10000000000.0f64);
    assert_eq!(c.im, 0.0f64);
}

#[test]
fn test_from_u128_ok() {
    let c = Complex::<f64>::from_u128(100000000000000000).unwrap();
    assert_eq!(c.re, 100000000000000000.0f64);
    assert_eq!(c.im, 0.0f64);
}

#[test]
fn test_from_f32_ok() {
    let c = Complex::<f64>::from_f32(10.5f32).unwrap();
    assert_eq!(c.re, 10.5f64);
    assert_eq!(c.im, 0.0f64);
}

#[test]
fn test_from_f32_nan() {
    let c = Complex::<f64>::from_f32(f32::nan());
    assert!(c.unwrap().re.is_nan());
    assert_eq!(c.unwrap().im, 0.0f64);
}

#[test]
fn test_from_f64_ok() {
    let c = Complex::<f64>::from_f64(10.5f64).unwrap();
    assert_eq!(c.re, 10.5f64);
    assert_eq!(c.im, 0.0f64);
}

#[test]
fn test_from_f64_nan() {
    let c = Complex::<f64>::from_f64(f64::nan());
    assert!(c.unwrap().re.is_nan());
    assert_eq!(c.unwrap().im, 0.0f64);
}
