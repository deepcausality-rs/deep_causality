/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::AdjustmentError;
use std::error::Error;

#[test]
fn test_adjustment_error_creation() {
    let error_msg = "test error message";
    let error = AdjustmentError::new(error_msg.to_string());
    assert_eq!(error.0, error_msg);
}

#[test]
fn test_adjustment_error_display() {
    let error_msg = "test error message";
    let error = AdjustmentError::new(error_msg.to_string());
    assert_eq!(
        format!("{error}"),
        format!("AdjustmentError: {}", error_msg)
    );
}

#[test]
fn test_adjustment_error_debug() {
    let error_msg = "test error message";
    let error = AdjustmentError::new(error_msg.to_string());
    assert_eq!(
        format!("{error:?}"),
        format!("AdjustmentError({:?})", error_msg)
    );
}

#[test]
fn test_adjustment_error_is_error() {
    let error = AdjustmentError::new("test".to_string());
    let is_error: &dyn Error = &error;
    assert!(is_error.is::<AdjustmentError>());
}
