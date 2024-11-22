// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::CausalityGraphError;
use std::error::Error;

#[test]
fn test_causality_graph_error_creation() {
    let error_msg = "test error message";
    let error = CausalityGraphError::new(error_msg.to_string());
    assert_eq!(error.0, error_msg);
}

#[test]
fn test_causality_graph_error_display() {
    let error_msg = "test error message";
    let error = CausalityGraphError::new(error_msg.to_string());
    assert_eq!(
        format!("{}", error),
        format!("CausalityGraphError: {}", error_msg)
    );
}

#[test]
fn test_causality_graph_error_debug() {
    let error_msg = "test error message";
    let error = CausalityGraphError::new(error_msg.to_string());
    assert_eq!(
        format!("{:?}", error),
        format!("CausalityGraphError({:?})", error_msg)
    );
}

#[test]
fn test_causality_graph_error_is_error() {
    let error = CausalityGraphError::new("test".to_string());
    let is_error: &dyn Error = &error;
    assert!(is_error.is::<CausalityGraphError>());
}
