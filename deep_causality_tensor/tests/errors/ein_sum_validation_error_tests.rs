/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::EinSumValidationError;
use std::error::Error;

#[test]
fn test_ein_sum_validation_error_invalid_number_of_children() {
    let error = EinSumValidationError::InvalidNumberOfChildren {
        expected: 2,
        found: 1,
    };
    assert_eq!(
        format!("{}", error),
        "EinSumValidationError: Invalid number of children. Expected 2, found 1"
    );
    assert_eq!(
        format!("{:?}", error),
        "InvalidNumberOfChildren { expected: 2, found: 1 }"
    );
    assert!(error.source().is_none());

    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
}

#[test]
fn test_ein_sum_validation_error_invalid_axes_specification() {
    let error = EinSumValidationError::InvalidAxesSpecification {
        message: "Axis 3 out of bounds".to_string(),
    };
    assert_eq!(
        format!("{}", error),
        "EinSumValidationError: Invalid axes specification: Axis 3 out of bounds"
    );
    assert_eq!(
        format!("{:?}", error),
        "InvalidAxesSpecification { message: \"Axis 3 out of bounds\" }"
    );
    assert!(error.source().is_none());

    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
}

#[test]
fn test_ein_sum_validation_error_unsupported_operation() {
    let error = EinSumValidationError::UnsupportedOperation {
        operation: "BatchMatMul".to_string(),
    };
    assert_eq!(
        format!("{}", error),
        "EinSumValidationError: Unsupported operation: BatchMatMul"
    );
    assert_eq!(
        format!("{:?}", error),
        "UnsupportedOperation { operation: \"BatchMatMul\" }"
    );
    assert!(error.source().is_none());

    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
}

#[test]
fn test_ein_sum_validation_error_shape_mismatch() {
    let error = EinSumValidationError::ShapeMismatch {
        message: "Dimensions 2x3 vs 3x2".to_string(),
    };
    assert_eq!(
        format!("{}", error),
        "EinSumValidationError: Shape mismatch: Dimensions 2x3 vs 3x2"
    );
    assert_eq!(
        format!("{:?}", error),
        "ShapeMismatch { message: \"Dimensions 2x3 vs 3x2\" }"
    );
    assert!(error.source().is_none());

    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
}

#[test]
fn test_ein_sum_validation_error_rank_mismatch() {
    let error = EinSumValidationError::RankMismatch {
        expected: 2,
        found: 3,
    };
    assert_eq!(
        format!("{}", error),
        "EinSumValidationError: Rank mismatch. Expected 2, found 3"
    );
    assert_eq!(
        format!("{:?}", error),
        "RankMismatch { expected: 2, found: 3 }"
    );
    assert!(error.source().is_none());

    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
}

#[test]
fn test_ein_sum_validation_error_partial_eq() {
    let error1 = EinSumValidationError::InvalidNumberOfChildren {
        expected: 2,
        found: 1,
    };
    let error2 = EinSumValidationError::InvalidNumberOfChildren {
        expected: 2,
        found: 1,
    };
    let error3 = EinSumValidationError::InvalidNumberOfChildren {
        expected: 3,
        found: 1,
    };

    assert_eq!(error1, error2);
    assert_ne!(error1, error3);

    let error4 = EinSumValidationError::ShapeMismatch {
        message: "test".to_string(),
    };
    assert_ne!(error1, error4);
}
