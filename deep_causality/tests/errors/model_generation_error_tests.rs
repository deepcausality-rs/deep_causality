/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::ModelGenerativeError;
use std::error::Error;

#[test]
fn test_model_generative_error_invalid_trigger() {
    let msg = "Invalid trigger message";
    let err = ModelGenerativeError::InvalidTrigger(msg.to_string());
    assert_eq!(format!("{err}"), format!("Invalid trigger: {}", msg));
    assert!(err.source().is_none());
}

#[test]
fn test_model_generative_error_invalid_time_kind() {
    let msg = "Invalid time kind message";
    let err = ModelGenerativeError::InvalidTimeKind(msg.to_string());
    assert_eq!(format!("{err}"), format!("Invalid time kind: {}", msg));
    assert!(err.source().is_none());
}

#[test]
fn test_model_generative_error_invalid_data_received_error() {
    let msg = "Invalid data received message";
    let err = ModelGenerativeError::InvalidDataReceivedError(msg.to_string());
    assert_eq!(
        format!("{err}"),
        format!("Invalid data received error: {}", msg)
    );
    assert!(err.source().is_none());
}

#[test]
fn test_model_generative_error_invalid_manual_intervention_error() {
    let msg = "Invalid manual intervention message";
    let err = ModelGenerativeError::InvalidManualInterventionError(msg.to_string());
    assert_eq!(
        format!("{err}"),
        format!("Invalid manual intervention error: {}", msg)
    );
    assert!(err.source().is_none());
}

#[test]
fn test_model_generative_error_internal_error() {
    let msg = "Internal error message";
    let err = ModelGenerativeError::InternalError(msg.to_string());
    assert_eq!(format!("{err}"), format!("Internal error: {}", msg));
    assert!(err.source().is_none());
}

#[test]
fn test_model_generative_error_user_defined_error() {
    let msg = "User defined error message";
    let err = ModelGenerativeError::UserDefinedError(msg.to_string());
    assert_eq!(format!("{err}"), format!("User defined error: {}", msg));
    assert!(err.source().is_none());
}
