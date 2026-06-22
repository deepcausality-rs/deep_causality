/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{ActionError, CausalityError};
use std::error::Error;

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
    assert_eq!(format!("{error}"), format!("ActionError: {}", error_msg));
}

#[test]
fn test_action_error_debug() {
    let error_msg = "test error message";
    let error = ActionError::new(error_msg.to_string());
    assert_eq!(
        format!("{error:?}"),
        format!("ActionError({:?})", error_msg)
    );
}

#[test]
fn test_action_error_is_error() {
    let error = ActionError::new("test".to_string());
    let is_error: &dyn Error = &error;
    assert!(is_error.is::<ActionError>());
}

#[test]
fn test_from_string() {
    let error_message = "This is an error from a string";
    let action_error: ActionError = String::from(error_message).into();
    assert_eq!(action_error.0, error_message);
}

#[test]
fn test_into_causality_error() {
    // `From<ActionError> for CausalityError` wraps the message in the
    // `ActionError` variant of the core error enum.
    let action_error = ActionError::new("boom".to_string());
    let causality_error: CausalityError = action_error.into();
    assert!(causality_error.to_string().contains("boom"));
}
