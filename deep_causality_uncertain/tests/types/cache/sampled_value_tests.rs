/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_uncertain::{
    FromSampledValue, IntoSampledValue, ProbabilisticType, SampledValue,
};

#[test]
fn test_sampled_value_float_display() {
    let val = SampledValue::Float(123.45);
    assert_eq!(format!("{}", val), "123.45");
}

#[test]
fn test_sampled_value_bool_display() {
    let val_true = SampledValue::Bool(true);
    assert_eq!(format!("{}", val_true), "true");

    let val_false = SampledValue::Bool(false);
    assert_eq!(format!("{}", val_false), "false");
}

#[test]
fn test_sampled_value_float_debug() {
    let val = SampledValue::Float(123.45);
    assert_eq!(format!("{:?}", val), "Float(123.45)");
}

#[test]
fn test_sampled_value_bool_debug() {
    let val_true = SampledValue::Bool(true);
    assert_eq!(format!("{:?}", val_true), "Bool(true)");

    let val_false = SampledValue::Bool(false);
    assert_eq!(format!("{:?}", val_false), "Bool(false)");
}

#[test]
fn test_sampled_value_float_clone() {
    let val = SampledValue::Float(123.45);
    let cloned_val = val;
    assert_eq!(val, cloned_val);
}

#[test]
fn test_sampled_value_bool_clone() {
    let val = SampledValue::Bool(true);
    let cloned_val = val;
    assert_eq!(val, cloned_val);
}

#[test]
fn test_sampled_value_float_copy() {
    let val = SampledValue::Float(123.45);
    let copied_val = val; // Copy happens implicitly
    assert_eq!(val, copied_val);
}

// Tests for IntoSampledValue implementation
#[test]
fn test_into_sampled_value_float() {
    let value = SampledValue::Float(42.0);
    assert_eq!(value.into_sampled_value(), SampledValue::Float(42.0));
}

#[test]
fn test_into_sampled_value_bool() {
    let value = SampledValue::Bool(true);
    assert_eq!(value.into_sampled_value(), SampledValue::Bool(true));
}

// Tests for FromSampledValue implementation
#[test]
fn test_from_sampled_value_float() {
    let value = SampledValue::Float(42.0);
    let result = SampledValue::from_sampled_value(value).unwrap();
    assert_eq!(result, SampledValue::Float(42.0));
}

#[test]
fn test_from_sampled_value_bool() {
    let value = SampledValue::Bool(true);
    let result = SampledValue::from_sampled_value(value).unwrap();
    assert_eq!(result, SampledValue::Bool(true));
}

// Test for ProbabilisticType implementation
#[test]
fn test_default_value() {
    assert_eq!(SampledValue::default_value(), SampledValue::Float(0.0));
}
