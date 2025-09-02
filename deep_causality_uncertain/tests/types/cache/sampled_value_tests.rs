/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_uncertain::SampledValue;

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
