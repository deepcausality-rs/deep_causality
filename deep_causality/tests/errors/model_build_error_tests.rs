/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::prelude::{ModelBuildError, ModelGenerativeError, ModelValidationError};
use std::error::Error;

#[test]
fn test_display_generation_failed() {
    let error = ModelBuildError::GenerationFailed(ModelGenerativeError::InternalError(
        "Failed to generate".to_string(),
    ));
    let expected = "The generation process failed: Internal error: Failed to generate";
    assert_eq!(format!("{error}"), expected);
}

#[test]
fn test_display_validation_failed() {
    let error = ModelBuildError::ValidationFailed(ModelValidationError::UnsupportedOperation {
        operation: "Invalid output".to_string(),
    });
    let expected = "The generative output was invalid for model construction: An unsupported operation was used during model construction: Invalid output. Only creation commands are allowed.";
    assert_eq!(format!("{error}"), expected);
}

#[test]
fn test_from_model_generative_error() {
    let gen_error = ModelGenerativeError::InternalError("Generation error".to_string());
    let build_error: ModelBuildError = gen_error.into();
    let expected = "The generation process failed: Internal error: Generation error";
    assert_eq!(format!("{build_error}"), expected);
}

#[test]
fn test_from_model_validation_error() {
    let val_error = ModelValidationError::MissingCreateCausaloid;
    let build_error: ModelBuildError = val_error.into();
    let expected = "The generative output was invalid for model construction: The generative output is missing the mandatory Causaloid creation command.";
    assert_eq!(format!("{build_error}"), expected);
}

#[test]
fn test_from_model_validation_error_duplicate_causaloid_id() {
    let val_error = ModelValidationError::DuplicateCausaloidID { id: 1 };
    let build_error: ModelBuildError = val_error.into();
    let expected = "The generative output was invalid for model construction: The generative output contains more than one 'CreateCausaloid' command, but exactly one is required. Duplicate ID: 1";
    assert_eq!(format!("{build_error}"), expected);
}

#[test]
fn test_from_model_validation_error_duplicate_context_id() {
    let val_error = ModelValidationError::DuplicateContextId { id: 1 };
    let build_error: ModelBuildError = val_error.into();
    let expected = "The generative output was invalid for model construction: The generative output contains a 'CreateContext' command with a duplicate ID: 1";
    assert_eq!(format!("{build_error}"), expected);
}

#[test]
fn test_from_model_validation_error_duplicate_contextoid_id() {
    let val_error = ModelValidationError::DuplicateContextoidId { id: 1 };
    let build_error: ModelBuildError = val_error.into();
    let expected = "The generative output was invalid for model construction: The generative output contains a 'CreateContextoid' command with a duplicate ID: 1";
    assert_eq!(format!("{build_error}"), expected);
}

#[test]
fn test_from_model_validation_error_target_context_not_found() {
    let val_error = ModelValidationError::TargetContextNotFound { id: 1 };
    let build_error: ModelBuildError = val_error.into();
    let expected = "The generative output was invalid for model construction: The generative output attempts to add a Contextoid to a Context (ID: 1) that was not created in the same generative step.";
    assert_eq!(format!("{build_error}"), expected);
}

#[test]
fn test_from_model_validation_error_unsupported_operation() {
    let val_error = ModelValidationError::UnsupportedOperation {
        operation: "DELETE".to_string(),
    };
    let build_error: ModelBuildError = val_error.into();
    let expected = "The generative output was invalid for model construction: An unsupported operation was used during model construction: DELETE. Only creation commands are allowed.";
    assert_eq!(format!("{build_error}"), expected);
}

#[test]
fn test_error_trait() {
    let error =
        ModelBuildError::GenerationFailed(ModelGenerativeError::InternalError("test".to_string()));
    let _source: Option<&(dyn Error + 'static)> = error.source();
}
