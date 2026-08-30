/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::MetricError;

#[test]
fn test_sign_convention_mismatch_display() {
    let error = MetricError::sign_convention_mismatch("expected East Coast, got West Coast");
    let msg = format!("{}", error);
    assert!(msg.contains("Sign convention mismatch"));
    assert!(msg.contains("expected East Coast"));
}

#[test]
fn test_invalid_dimension_display() {
    let error = MetricError::invalid_dimension("dimension cannot be zero");
    let msg = format!("{}", error);
    assert!(msg.contains("Invalid dimension"));
    assert!(msg.contains("cannot be zero"));
}

#[test]
fn test_validation_failed_display() {
    let error = MetricError::validation_failed("sign must be +1, -1, or 0");
    let msg = format!("{}", error);
    assert!(msg.contains("validation failed"));
    assert!(msg.contains("sign must be"));
}

#[test]
fn test_conversion_error_display() {
    let error = MetricError::conversion_error("cannot convert non-Lorentzian metric");
    let msg = format!("{}", error);
    assert!(msg.contains("Conversion error"));
    assert!(msg.contains("non-Lorentzian"));
}

#[test]
fn test_error_equality() {
    let e1 = MetricError::invalid_dimension("test");
    let e2 = MetricError::invalid_dimension("test");
    let e3 = MetricError::invalid_dimension("other");

    assert_eq!(e1, e2);
    assert_ne!(e1, e3);
}

#[test]
fn test_error_clone() {
    let error = MetricError::sign_convention_mismatch("test");
    let cloned = error.clone();
    assert_eq!(error, cloned);
}

#[test]
fn test_error_debug() {
    let error = MetricError::validation_failed("test");
    let debug = format!("{:?}", error);
    assert!(debug.contains("ValidationFailed"));
}
