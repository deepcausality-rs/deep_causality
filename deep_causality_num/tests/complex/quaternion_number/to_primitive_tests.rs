/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Quaternion;
use deep_causality_num::ToPrimitive;

#[test]
fn test_to_isize() {
    let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    assert_eq!(q.to_isize(), Some(123));
    let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    assert_eq!(q_nan.to_isize(), None);
    let q_inf = Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_inf.to_isize(), None);
    let q_neg_inf = Quaternion::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_neg_inf.to_isize(), None);
}

#[test]
fn test_to_i8() {
    let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    assert_eq!(q.to_i8(), Some(123));
    let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    assert_eq!(q_nan.to_i8(), None);
    let q_inf = Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_inf.to_i8(), None);
    let q_neg_inf = Quaternion::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_neg_inf.to_i8(), None);
}

#[test]
fn test_to_i16() {
    let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    assert_eq!(q.to_i16(), Some(123));
    let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    assert_eq!(q_nan.to_i16(), None);
    let q_inf = Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_inf.to_i16(), None);
    let q_neg_inf = Quaternion::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_neg_inf.to_i16(), None);
}

#[test]
fn test_to_i32() {
    let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    assert_eq!(q.to_i32(), Some(123));
    let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    assert_eq!(q_nan.to_i32(), None);
    let q_inf = Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_inf.to_i32(), None);
    let q_neg_inf = Quaternion::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_neg_inf.to_i32(), None);
}

#[test]
fn test_to_i64() {
    let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    assert_eq!(q.to_i64(), Some(123));
    let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    assert_eq!(q_nan.to_i64(), None);
    let q_inf = Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_inf.to_i64(), None);
    let q_neg_inf = Quaternion::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_neg_inf.to_i64(), None);
}

#[test]
fn test_to_i128() {
    let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    assert_eq!(q.to_i128(), Some(123));
    let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    assert_eq!(q_nan.to_i128(), None);
    let q_inf = Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_inf.to_i128(), None);
    let q_neg_inf = Quaternion::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_neg_inf.to_i128(), None);
}

#[test]
fn test_to_usize() {
    let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    assert_eq!(q.to_usize(), Some(123));
    let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    assert_eq!(q_nan.to_usize(), None);
    let q_inf = Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_inf.to_usize(), None);
    let q_neg_inf = Quaternion::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_neg_inf.to_usize(), None);
}

#[test]
fn test_to_u8() {
    let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    assert_eq!(q.to_u8(), Some(123));
    let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    assert_eq!(q_nan.to_u8(), None);
    let q_inf = Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_inf.to_u8(), None);
    let q_neg_inf = Quaternion::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_neg_inf.to_u8(), None);
}

#[test]
fn test_to_u16() {
    let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    assert_eq!(q.to_u16(), Some(123));
    let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    assert_eq!(q_nan.to_u16(), None);
    let q_inf = Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_inf.to_u16(), None);
    let q_neg_inf = Quaternion::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_neg_inf.to_u16(), None);
}

#[test]
fn test_to_u32() {
    let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    assert_eq!(q.to_u32(), Some(123));
    let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    assert_eq!(q_nan.to_u32(), None);
    let q_inf = Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_inf.to_u32(), None);
    let q_neg_inf = Quaternion::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_neg_inf.to_u32(), None);
}

#[test]
fn test_to_u64() {
    let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    assert_eq!(q.to_u64(), Some(123));
    let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    assert_eq!(q_nan.to_u64(), None);
    let q_inf = Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_inf.to_u64(), None);
    let q_neg_inf = Quaternion::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_neg_inf.to_u64(), None);
}

#[test]
fn test_to_u128() {
    let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    assert_eq!(q.to_u128(), Some(123));
    let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    assert_eq!(q_nan.to_u128(), None);
    let q_inf = Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_inf.to_u128(), None);
    let q_neg_inf = Quaternion::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_neg_inf.to_u128(), None);
}

#[test]
fn test_to_f32() {
    let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    assert_eq!(q.to_f32(), Some(123.45_f32));
    let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    assert!(q_nan.to_f32().unwrap().is_nan());
    let q_inf = Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_inf.to_f32(), Some(f32::INFINITY));
    let q_neg_inf = Quaternion::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_neg_inf.to_f32(), Some(f32::NEG_INFINITY));
}

#[test]
fn test_to_f64() {
    let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    assert_eq!(q.to_f64(), Some(123.45_f64));
    let q_nan = Quaternion::new(f64::NAN, 0.0, 0.0, 0.0);
    assert!(q_nan.to_f64().unwrap().is_nan());
    let q_inf = Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_inf.to_f64(), Some(f64::INFINITY));
    let q_neg_inf = Quaternion::new(f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    assert_eq!(q_neg_inf.to_f64(), Some(f64::NEG_INFINITY));
}
