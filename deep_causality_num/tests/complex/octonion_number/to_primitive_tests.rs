/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Float, Octonion, ToPrimitive};

#[test]
fn test_to_isize_ok() {
    let o = Octonion::new(10.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_isize(), Some(10));
}

#[test]
fn test_to_isize_fail_fraction() {
    let o = Octonion::new(10.5, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    // The default f64::to_isize() truncates, so it returns Some(10)
    assert_eq!(o.to_isize(), Some(10));
}

#[test]
fn test_to_isize_fail_overflow() {
    let o = Octonion::new(f64::MAX, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_isize(), None);
}

#[test]
fn test_to_isize_nan() {
    let o = Octonion::new(f64::nan(), 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_isize(), None);
}

#[test]
fn test_to_i8_ok() {
    let o = Octonion::new(10.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_i8(), Some(10));
}

#[test]
fn test_to_i8_fail_overflow() {
    let o = Octonion::new(200.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_i8(), None);
}

#[test]
fn test_to_i16_ok() {
    let o = Octonion::new(1000.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_i16(), Some(1000));
}

#[test]
fn test_to_i16_fail_overflow() {
    let o = Octonion::new(f64::MAX, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_i16(), None);
}

#[test]
fn test_to_i32_ok() {
    let o = Octonion::new(100000.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_i32(), Some(100000));
}

#[test]
fn test_to_i32_fail_overflow() {
    let o = Octonion::new(f64::MAX, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_i32(), None);
}

#[test]
fn test_to_i64_ok() {
    let o = Octonion::new(10000000000.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_i64(), Some(10000000000));
}

#[test]
fn test_to_i64_fail_overflow() {
    let o = Octonion::new(f64::MAX, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_i64(), None);
}

#[test]
fn test_to_i128_ok() {
    let o = Octonion::new(100000000000000000.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_i128(), Some(100000000000000000));
}

#[test]
fn test_to_i128_fail_overflow() {
    let o = Octonion::new(f64::MAX, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_i128(), None);
}

#[test]
fn test_to_usize_ok() {
    let o = Octonion::new(10.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_usize(), Some(10));
}

#[test]
fn test_to_usize_fail_negative() {
    let o = Octonion::new(-10.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_usize(), None);
}

#[test]
fn test_to_u8_ok() {
    let o = Octonion::new(10.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_u8(), Some(10));
}

#[test]
fn test_to_u8_fail_overflow() {
    let o = Octonion::new(300.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_u8(), None);
}

#[test]
fn test_to_u16_ok() {
    let o = Octonion::new(1000.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_u16(), Some(1000));
}

#[test]
fn test_to_u16_fail_overflow() {
    let o = Octonion::new(f64::MAX, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_u16(), None);
}

#[test]
fn test_to_u32_ok() {
    let o = Octonion::new(100000.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_u32(), Some(100000));
}

#[test]
fn test_to_u32_fail_overflow() {
    let o = Octonion::new(f64::MAX, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_u32(), None);
}

#[test]
fn test_to_u64_ok() {
    let o = Octonion::new(10000000000.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_u64(), Some(10000000000));
}

#[test]
fn test_to_u64_fail_overflow() {
    let o = Octonion::new(f64::MAX, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_u64(), None);
}

#[test]
fn test_to_u128_ok() {
    let o = Octonion::new(100000000000000000.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_u128(), Some(100000000000000000));
}

#[test]
fn test_to_u128_fail_overflow() {
    let o = Octonion::new(f64::MAX, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_u128(), None);
}

#[test]
fn test_to_f32_ok() {
    let o = Octonion::new(10.5, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_f32(), Some(10.5f32));
}

#[test]
fn test_to_f32_nan() {
    let o = Octonion::new(f64::nan(), 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert!(o.to_f32().unwrap().is_nan());
}

#[test]
fn test_to_f64_ok() {
    let o = Octonion::new(10.5, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(o.to_f64(), Some(10.5));
}

#[test]
fn test_to_f64_nan() {
    let o = Octonion::new(f64::nan(), 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert!(o.to_f64().unwrap().is_nan());
}
