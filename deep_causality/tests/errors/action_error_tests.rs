// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::error::Error;
use deep_causality::prelude::ActionError;

#[test]
fn test_action_error_creation() {
    let error_msg = "test error message";
    let error = ActionError::new(error_msg.to_string());
    assert_eq!(error.0, error_msg);
}

#[test]
fn test_action_error_display() {
    let error_msg = "test error message";
    let error = ActionError::new(error_msg.to_string());
    assert_eq!(format!("{}", error), format!("ActionError: {}", error_msg));
}

#[test]
fn test_action_error_debug() {
    let error_msg = "test error message";
    let error = ActionError::new(error_msg.to_string());
    assert_eq!(format!("{:?}", error), format!("ActionError({:?})", error_msg));
}

#[test]
fn test_action_error_is_error() {
    let error = ActionError::new("test".to_string());
    let is_error: &dyn Error = &error;
    assert!(is_error.is::<ActionError>());
}
