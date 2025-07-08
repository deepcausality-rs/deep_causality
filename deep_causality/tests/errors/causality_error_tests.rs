/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{CausalityError, CausalityGraphError};
use std::error::Error;
use ultragraph::GraphError;

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

#[test]
fn test_from_graph_error() {
    // Create an instance of the external error type.
    let graph_err = GraphError::NodeNotFound(42);
    // Perform the conversion.
    let causality_err = CausalityError::from(graph_err);
    // Check that the Display output is formatted correctly.
    let expected_msg = "CausalityError: Graph operation failed: Node with index 42 not found; it may be out of bounds or have been removed.";
    assert_eq!(causality_err.to_string(), expected_msg);
}

#[test]
fn test_from_causality_graph_error() {
    // Create an instance of the internal, more specific error type.
    let causal_graph_err = CausalityGraphError("test causal graph error".to_string());
    // Perform the conversion.
    let causality_err = CausalityError::from(causal_graph_err);
    // Check that the Display output is formatted correctly.
    let expected_msg = "CausalityError: CausalityGraphError: test causal graph error";
    assert_eq!(causality_err.to_string(), expected_msg);
}
