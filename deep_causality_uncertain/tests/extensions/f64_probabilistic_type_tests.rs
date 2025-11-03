/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_uncertain::{
    FromSampledValue, IntoSampledValue, ProbabilisticType, SampledValue, UncertainError,
};

// Test for IntoSampledValue for f64
#[test]
fn test_f64_into_sampled_value() {
    let val = 42.0;
    assert_eq!(val.into_sampled_value(), SampledValue::Float(42.0));
}

// Tests for FromSampledValue for f64
#[test]
fn test_f64_from_sampled_value_float() {
    let sampled = SampledValue::Float(42.0);
    let result = f64::from_sampled_value(sampled);
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val, 42.0);
}

#[test]
fn test_f64_from_sampled_value_bool_err() {
    let sampled = SampledValue::Bool(true);
    let result = f64::from_sampled_value(sampled);
    // Ensure the correct error type and message is returned
    assert!(matches!(
        result,
        Err(UncertainError::UnsupportedTypeError(_))
    ));
    if let Err(UncertainError::UnsupportedTypeError(msg)) = result {
        assert_eq!(msg, "Expected f64 SampledValue");
    }
}

// Test for ProbabilisticType for f64
#[test]
fn test_f64_default_value() {
    assert_eq!(f64::default_value(), 0.0);
}
