/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{CausalTensorError, EinSumValidationError};
use std::error::Error;

#[test]
fn test_error_display_and_debug() {
    let test_cases = [
        (
            CausalTensorError::ShapeMismatch,
            "CausalTensorError: Shape mismatch error",
            "ShapeMismatch",
        ),
        (
            CausalTensorError::DimensionMismatch,
            "CausalTensorError: Dimension mismatch error",
            "DimensionMismatch",
        ),
        (
            CausalTensorError::DivisionByZero,
            "CausalTensorError: Division by zero error",
            "DivisionByZero",
        ),
        (
            CausalTensorError::AxisOutOfBounds,
            "CausalTensorError: Axis out of bounds error",
            "AxisOutOfBounds",
        ),
        (
            CausalTensorError::EmptyTensor,
            "CausalTensorError: Empty tensor error",
            "EmptyTensor",
        ),
        (
            CausalTensorError::InvalidOperation,
            "CausalTensorError: Invalid operation error",
            "InvalidOperation",
        ),
        (
            CausalTensorError::UnorderableValue,
            "CausalTensorError: Unorderable value encountered",
            "UnorderableValue",
        ),
        (
            CausalTensorError::InvalidParameter("Invalid input".to_string()),
            "CausalTensorError: Invalid parameter: Invalid input",
            "InvalidParameter(\"Invalid input\")",
        ),
        (
            CausalTensorError::EinSumError(EinSumValidationError::ShapeMismatch {
                message: "Test EinSum Shape Mismatch".to_string(),
            }),
            "CausalTensorError: EinSumError: EinSumValidationError: Shape mismatch: Test EinSum Shape Mismatch",
            "EinSumError(ShapeMismatch { message: \"Test EinSum Shape Mismatch\" })",
        ),
    ];

    for (err, display_msg, debug_msg) in &test_cases {
        assert_eq!(err.to_string(), *display_msg);
        assert_eq!(format!("{:?}", err), *debug_msg);
    }
}

#[test]
fn test_error_equality() {
    assert_eq!(
        CausalTensorError::ShapeMismatch,
        CausalTensorError::ShapeMismatch
    );
    assert_ne!(
        CausalTensorError::ShapeMismatch,
        CausalTensorError::DimensionMismatch
    );
    assert_ne!(
        CausalTensorError::DimensionMismatch,
        CausalTensorError::DivisionByZero
    );
    assert_ne!(
        CausalTensorError::DimensionMismatch,
        CausalTensorError::AxisOutOfBounds
    );
    assert_ne!(
        CausalTensorError::AxisOutOfBounds,
        CausalTensorError::EmptyTensor
    );
    assert_ne!(
        CausalTensorError::EmptyTensor,
        CausalTensorError::InvalidOperation
    );
    assert_ne!(
        CausalTensorError::InvalidOperation,
        CausalTensorError::UnorderableValue
    );
    assert_eq!(
        CausalTensorError::InvalidParameter("test".to_string()),
        CausalTensorError::InvalidParameter("test".to_string())
    );
    assert_ne!(
        CausalTensorError::InvalidParameter("test1".to_string()),
        CausalTensorError::InvalidParameter("test2".to_string())
    );
    assert_eq!(
        CausalTensorError::EinSumError(EinSumValidationError::ShapeMismatch {
            message: "Test".to_string()
        }),
        CausalTensorError::EinSumError(EinSumValidationError::ShapeMismatch {
            message: "Test".to_string()
        })
    );
    assert_ne!(
        CausalTensorError::EinSumError(EinSumValidationError::ShapeMismatch {
            message: "Test1".to_string()
        }),
        CausalTensorError::EinSumError(EinSumValidationError::RankMismatch {
            expected: 1,
            found: 2
        })
    );
}

#[test]
fn test_error_trait_source() {
    let errors = [
        CausalTensorError::ShapeMismatch,
        CausalTensorError::DimensionMismatch,
        CausalTensorError::DivisionByZero,
        CausalTensorError::AxisOutOfBounds,
        CausalTensorError::EmptyTensor,
        CausalTensorError::InvalidOperation,
        CausalTensorError::UnorderableValue,
        CausalTensorError::InvalidParameter("test".to_string()),
        CausalTensorError::EinSumError(EinSumValidationError::ShapeMismatch {
            message: "Test".to_string(),
        }),
    ];

    for err in &errors {
        assert!(err.source().is_none());
    }
}
