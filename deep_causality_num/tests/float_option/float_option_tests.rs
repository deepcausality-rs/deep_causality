/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::FloatOption;

#[test]
fn test_f32_to_option_f32() {
    let val: f32 = 42.0;
    assert_eq!(val.to_option(), Some(42.0f32));

    let val_neg: f32 = -42.0;
    assert_eq!(val_neg.to_option(), Some(-42.0f32));

    let val_zero: f32 = 0.0;
    assert_eq!(val_zero.to_option(), Some(0.0f32));

    let val_nan: f32 = f32::NAN;
    assert_eq!(val_nan.to_option(), None);
}

#[test]
fn test_f64_to_option_f64() {
    let val: f64 = 42.0;
    assert_eq!(val.to_option(), Some(42.0f64));

    let val_neg: f64 = -42.0;
    assert_eq!(val_neg.to_option(), Some(-42.0f64));

    let val_zero: f64 = 0.0;
    assert_eq!(val_zero.to_option(), Some(0.0f64));

    let val_nan: f64 = f64::NAN;
    assert_eq!(val_nan.to_option(), None);
}

#[test]
fn test_option_f32_to_option_f32() {
    let val: Option<f32> = Some(42.0);
    assert_eq!(val.to_option(), Some(42.0f32));

    let val_neg: Option<f32> = Some(-42.0);
    assert_eq!(val_neg.to_option(), Some(-42.0f32));

    let val_zero: Option<f32> = Some(0.0);
    assert_eq!(val_zero.to_option(), Some(0.0f32));

    let val_nan: Option<f32> = Some(f32::NAN);
    assert_eq!(val_nan.to_option(), None);

    let val_none: Option<f32> = None;
    assert_eq!(val_none.to_option(), None);
}

#[test]
fn test_option_f64_to_option_f64() {
    let val: Option<f64> = Some(42.0);
    assert_eq!(val.to_option(), Some(42.0f64));

    let val_neg: Option<f64> = Some(-42.0);
    assert_eq!(val_neg.to_option(), Some(-42.0f64));

    let val_zero: Option<f64> = Some(0.0);
    assert_eq!(val_zero.to_option(), Some(0.0f64));

    let val_nan: Option<f64> = Some(f64::NAN);
    assert_eq!(val_nan.to_option(), None);

    let val_none: Option<f64> = None;
    assert_eq!(val_none.to_option(), None);
}
