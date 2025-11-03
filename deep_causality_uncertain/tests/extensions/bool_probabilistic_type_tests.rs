/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_uncertain::{
    FromSampledValue, IntoSampledValue, ProbabilisticType, SampledValue, UncertainError,
};

// Tests for IntoSampledValue for bool
#[test]
fn test_bool_into_sampled_value_true() {
    let val = true;
    assert_eq!(val.into_sampled_value(), SampledValue::Bool(true));
}

#[test]
fn test_bool_into_sampled_value_false() {
    let val = false;
    assert_eq!(val.into_sampled_value(), SampledValue::Bool(false));
}

// Tests for FromSampledValue for bool
#[test]
fn test_bool_from_sampled_value_bool() {
    let sampled = SampledValue::Bool(true);
    let result = bool::from_sampled_value(sampled);
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(val);
}

#[test]
fn test_bool_from_sampled_value_float_err() {
    let sampled = SampledValue::Float(0.42);
    let result = bool::from_sampled_value(sampled);
    // Ensure the correct error type and message is returned
    assert!(matches!(
        result,
        Err(UncertainError::UnsupportedTypeError(_))
    ));
    if let Err(UncertainError::UnsupportedTypeError(msg)) = result {
        assert_eq!(msg, "Expected bool SampledValue");
    }
}

// Test for ProbabilisticType for bool
#[test]
fn test_bool_default_value() {
    assert!(!bool::default_value());
}
