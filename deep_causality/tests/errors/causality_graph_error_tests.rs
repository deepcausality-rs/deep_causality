/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{CausalityError, CausalityGraphError};
use std::error::Error;
use ultragraph::GraphError;

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
        format!("{error}"),
        format!("CausalityGraphError: {}", error_msg)
    );
}

#[test]
fn test_causality_graph_error_debug() {
    let error_msg = "test error message";
    let error = CausalityGraphError::new(error_msg.to_string());
    assert_eq!(
        format!("{error:?}"),
        format!("CausalityGraphError({:?})", error_msg)
    );
}

#[test]
fn test_causality_graph_error_is_error() {
    let error = CausalityGraphError::new("test".to_string());
    let is_error: &dyn Error = &error;
    assert!(is_error.is::<CausalityGraphError>());
}

#[test]
fn test_from_graph_error_conversion() {
    // 1. Arrange: Create an instance of the source error type.
    let source_error = GraphError::GraphIsFrozen;
    let expected_message = source_error.to_string();

    // 2. Act: Convert the source error into the target error using `.into()`.
    let causality_error: CausalityGraphError = source_error.into();

    // 3. Assert: Verify that the converted error contains the correct message.
    assert_eq!(causality_error.0, expected_message);
    assert_eq!(
        causality_error.to_string(),
        format!("CausalityGraphError: {expected_message}")
    );
}

#[test]
fn test_from_causality_error_conversion() {
    // 1. Arrange: Create an instance of the source error type.
    let source_error = CausalityError::new(deep_causality::CausalityErrorEnum::Custom(
        "inner error".to_string(),
    ));
    // The `From` impl uses the `Display` format of the source error.
    let expected_message = source_error.to_string();

    // 2. Act: Convert the source error into the target error using `.into()`.
    let causality_graph_error: CausalityGraphError = source_error.into();

    // 3. Assert: Verify that the converted error contains the correct message.
    assert_eq!(causality_graph_error.0, expected_message);
    assert_eq!(
        causality_graph_error.to_string(),
        format!("CausalityGraphError: {expected_message}")
    );
}
