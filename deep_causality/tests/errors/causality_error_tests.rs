/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::CausalityError;
use std::error::Error;

#[test]
fn test_causality_error_creation() {
    let error_msg = "test error message";
    let error = CausalityError::new(error_msg.to_string());
    assert_eq!(error.0, error_msg);
}

#[test]
fn test_causality_error_display() {
    let error_msg = "test error message";
    let error = CausalityError::new(error_msg.to_string());
    assert_eq!(format!("{error}"), format!("CausalityError: {}", error_msg));
}

#[test]
fn test_causality_error_debug() {
    let error_msg = "test error message";
    let error = CausalityError::new(error_msg.to_string());
    assert_eq!(
        format!("{error:?}"),
        format!("CausalityError({:?})", error_msg)
    );
}

#[test]
fn test_causality_error_is_error() {
    let error = CausalityError::new("test".to_string());
    let is_error: &dyn Error = &error;
    assert!(is_error.is::<CausalityError>());
}
