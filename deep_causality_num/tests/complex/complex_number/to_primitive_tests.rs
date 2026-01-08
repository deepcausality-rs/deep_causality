/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Complex, Float, ToPrimitive};

#[test]
fn test_to_isize_ok() {
    let c = Complex::new(10.0f64, 5.0f64);
    assert_eq!(c.to_isize(), Some(10));
}

#[test]
fn test_to_isize_fail_fraction() {
    let c = Complex::new(10.5f64, 5.0f64);
    // The default f64::to_isize() truncates, so it returns Some(10)
    assert_eq!(c.to_isize(), Some(10));
}

#[test]
fn test_to_isize_fail_overflow() {
    let c = Complex::new(f64::MAX, 5.0f64);
    assert_eq!(c.to_isize(), None);
}

#[test]
fn test_to_isize_nan() {
    let c = Complex::new(f64::nan(), 5.0f64);
    assert_eq!(c.to_isize(), None);
}

#[test]
fn test_to_i8_ok() {
    let c = Complex::new(10.0f64, 5.0f64);
    assert_eq!(c.to_i8(), Some(10));
}

#[test]
fn test_to_i8_fail_overflow() {
    let c = Complex::new(200.0f64, 5.0f64);
    assert_eq!(c.to_i8(), None);
}

#[test]
fn test_to_i16_ok() {
    let c = Complex::new(1000.0f64, 5.0f64);
    assert_eq!(c.to_i16(), Some(1000));
}

#[test]
fn test_to_i16_fail_overflow() {
    let c = Complex::new(f64::MAX, 5.0f64);
    assert_eq!(c.to_i16(), None);
}

#[test]
fn test_to_i32_ok() {
    let c = Complex::new(100000.0f64, 5.0f64);
    assert_eq!(c.to_i32(), Some(100000));
}

#[test]
fn test_to_i32_fail_overflow() {
    let c = Complex::new(f64::MAX, 5.0f64);
    assert_eq!(c.to_i32(), None);
}

#[test]
fn test_to_i64_ok() {
    let c = Complex::new(10000000000.0f64, 5.0f64);
    assert_eq!(c.to_i64(), Some(10000000000));
}

#[test]
fn test_to_i64_fail_overflow() {
    let c = Complex::new(f64::MAX, 5.0f64);
    assert_eq!(c.to_i64(), None);
}

#[test]
fn test_to_i128_ok() {
    let c = Complex::new(100000000000000000.0f64, 5.0f64);
    assert_eq!(c.to_i128(), Some(100000000000000000));
}

#[test]
fn test_to_i128_fail_overflow() {
    let c = Complex::new(f64::MAX, 5.0f64);
    assert_eq!(c.to_i128(), None);
}

#[test]
fn test_to_usize_ok() {
    let c = Complex::new(10.0f64, 5.0f64);
    assert_eq!(c.to_usize(), Some(10));
}

#[test]
fn test_to_usize_fail_negative() {
    let c = Complex::new(-10.0f64, 5.0f64);
    assert_eq!(c.to_usize(), None);
}

#[test]
fn test_to_u8_ok() {
    let c = Complex::new(10.0f64, 5.0f64);
    assert_eq!(c.to_u8(), Some(10));
}

#[test]
fn test_to_u8_fail_overflow() {
    let c = Complex::new(300.0f64, 5.0f64);
    assert_eq!(c.to_u8(), None);
}

#[test]
fn test_to_u16_ok() {
    let c = Complex::new(1000.0f64, 5.0f64);
    assert_eq!(c.to_u16(), Some(1000));
}

#[test]
fn test_to_u16_fail_overflow() {
    let c = Complex::new(f64::MAX, 5.0f64);
    assert_eq!(c.to_u16(), None);
}

#[test]
fn test_to_u32_ok() {
    let c = Complex::new(100000.0f64, 5.0f64);
    assert_eq!(c.to_u32(), Some(100000));
}

#[test]
fn test_to_u32_fail_overflow() {
    let c = Complex::new(f64::MAX, 5.0f64);
    assert_eq!(c.to_u32(), None);
}

#[test]
fn test_to_u64_ok() {
    let c = Complex::new(10000000000.0f64, 5.0f64);
    assert_eq!(c.to_u64(), Some(10000000000));
}

#[test]
fn test_to_u64_fail_overflow() {
    let c = Complex::new(f64::MAX, 5.0f64);
    assert_eq!(c.to_u64(), None);
}

#[test]
fn test_to_u128_ok() {
    let c = Complex::new(100000000000000000.0f64, 5.0f64);
    assert_eq!(c.to_u128(), Some(100000000000000000));
}

#[test]
fn test_to_u128_fail_overflow() {
    let c = Complex::new(f64::MAX, 5.0f64);
    assert_eq!(c.to_u128(), None);
}

#[test]
fn test_to_f32_ok() {
    let c = Complex::new(10.5f64, 5.0f64);
    assert_eq!(c.to_f32(), Some(10.5f32));
}

#[test]
fn test_to_f32_nan() {
    let c = Complex::new(f64::nan(), 5.0f64);
    assert!(c.to_f32().unwrap().is_nan());
}

#[test]
fn test_to_f64_ok() {
    let c = Complex::new(10.5f64, 5.0f64);
    assert_eq!(c.to_f64(), Some(10.5f64));
}

#[test]
fn test_to_f64_nan() {
    let c = Complex::new(f64::nan(), 5.0f64);
    assert!(c.to_f64().unwrap().is_nan());
}
