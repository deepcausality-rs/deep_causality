/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};

use deep_causality::prelude::{ModelBuildError, ModelGenerativeError, ModelValidationError};

fn get_error_source(err: &dyn Error) -> String {
    err.source().unwrap().to_string()
}

#[test]
fn test_display() {
    let err =
        ModelBuildError::GenerationFailed(ModelGenerativeError::InternalError("test".to_string()));
    assert_eq!(
        "The generation process failed: Internal error: test",
        err.to_string()
    );

    let err = ModelBuildError::ValidationFailed(ModelValidationError::BaseContextNotFound);
    assert_eq!(
        "The generative output was invalid for model construction: Cannot perform operation because the base context has not been created",
        err.to_string()
    );
}

#[test]
fn test_source() {
    let err =
        ModelBuildError::GenerationFailed(ModelGenerativeError::InternalError("test".to_string()));
    assert_eq!("Internal error: test", get_error_source(&err));

    let err = ModelBuildError::ValidationFailed(ModelValidationError::BaseContextNotFound);
    assert_eq!(
        "Cannot perform operation because the base context has not been created",
        get_error_source(&err)
    );
}

#[test]
fn test_from_model_generative_error() {
    let err = ModelGenerativeError::InternalError("test".to_string());
    let model_build_err: ModelBuildError = err.into();

    assert_eq!(
        "The generation process failed: Internal error: test",
        model_build_err.to_string()
    );
}

#[test]
fn test_from_model_validation_error() {
    let err = ModelValidationError::BaseContextNotFound;
    let model_build_err: ModelBuildError = err.into();

    assert_eq!(
        "The generative output was invalid for model construction: Cannot perform operation because the base context has not been created",
        model_build_err.to_string()
    );
}

#[test]
fn test_debug_clone_hash_eq_partial_eq() {
    let err1 =
        ModelBuildError::GenerationFailed(ModelGenerativeError::InternalError("test".to_string()));
    let err2 = err1.clone();

    // Check that the cloned error is equal to the original
    assert_eq!(err1, err2);

    // Check that the debug format is as expected
    assert_eq!(
        format!("{err1:?}"),
        "GenerationFailed(InternalError(\"test\"))"
    );

    // Check that the hash of the two errors is the same
    let mut hasher1 = DefaultHasher::new();
    err1.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    err2.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    assert_eq!(hash1, hash2);
}
