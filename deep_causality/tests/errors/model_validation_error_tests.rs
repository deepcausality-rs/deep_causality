/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::ModelValidationError;
use std::error::Error;

#[test]
fn test_missing_create_causaloid() {
    let err = ModelValidationError::MissingCreateCausaloid;
    assert_eq!(
        format!("{err}"),
        "The generative output is missing the mandatory Causaloid creation command."
    );
    assert!(err.source().is_none());
}

#[test]
fn test_duplicate_causaloid_id() {
    let err = ModelValidationError::DuplicateCausaloidID { id: 42 };
    assert_eq!(format!("{err}"), "Duplicate Causaloid ID found: 42");
    assert!(err.source().is_none());
}

#[test]
fn test_target_causaloid_not_found() {
    let err = ModelValidationError::TargetCausaloidNotFound { id: 42 };
    assert_eq!(format!("{err}"), "Target Causaloid with ID 42 not found");
    assert!(err.source().is_none());
}

#[test]
fn test_base_context_not_found() {
    let err = ModelValidationError::BaseContextNotFound;
    assert_eq!(
        format!("{err}"),
        "Cannot perform operation because the base context has not been created"
    );
    assert!(err.source().is_none());
}

#[test]
fn test_duplicate_context_id() {
    let err = ModelValidationError::DuplicateContextId { id: 42 };
    assert_eq!(format!("{err}"), "Duplicate Context ID found: 42");
    assert!(err.source().is_none());
}

#[test]
fn test_target_context_not_found() {
    let err = ModelValidationError::TargetContextNotFound { id: 42 };
    assert_eq!(format!("{err}"), "Target Context with ID 42 not found");
    assert!(err.source().is_none());
}

#[test]
fn test_duplicate_extra_context_id() {
    let err = ModelValidationError::DuplicateExtraContextId { id: 42 };
    assert_eq!(format!("{err}"), "Duplicate Extra Context ID found: 42");
    assert!(err.source().is_none());
}

#[test]
fn test_target_contextoid_not_found() {
    let err = ModelValidationError::TargetContextoidNotFound { id: 42 };
    assert_eq!(format!("{err}"), "Target Contextoid with ID 42 not found");
    assert!(err.source().is_none());
}

#[test]
fn test_duplicate_contextoid_id() {
    let err = ModelValidationError::DuplicateContextoidId { id: 42 };
    assert_eq!(format!("{err}"), "Duplicate Contextoid ID found: 42");
    assert!(err.source().is_none());
}

#[test]
fn test_add_contextoid_id() {
    let err = ModelValidationError::AddContextoidError {
        err: "error".to_string(),
    };
    assert_eq!(format!("{err}"), "Error adding Contextoid: error");
    assert!(err.source().is_none());
}

#[test]
fn test_unsupported_operation() {
    let err = ModelValidationError::UnsupportedOperation {
        operation: "test".to_string(),
    };
    assert_eq!(format!("{err}"), "Unsupported operation: test");
    assert!(err.source().is_none());
}

#[test]
fn test_update_node_error() {
    let err = ModelValidationError::UpdateNodeError {
        err: "failed to update".to_string(),
    };
    assert_eq!(format!("{err}"), "Error updating node: failed to update");
    assert!(err.source().is_none());
}

#[test]
fn test_remove_node_error() {
    let err = ModelValidationError::RemoveNodeError {
        err: "failed to remove".to_string(),
    };
    assert_eq!(format!("{err}"), "Error removing node: failed to remove");
    assert!(err.source().is_none());
}

#[test]
fn test_interpreter_error() {
    let err = ModelValidationError::InterpreterError {
        reason: "invalid syntax".to_string(),
    };
    assert_eq!(format!("{err}"), "Interpreter error: invalid syntax");
    assert!(err.source().is_none());
}
