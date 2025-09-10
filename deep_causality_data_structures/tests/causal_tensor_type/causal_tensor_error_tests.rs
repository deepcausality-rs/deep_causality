/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_data_structures::causal_tensor_type::error::CausalTensorError;
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
}

#[test]
fn test_error_trait_source() {
    let errors = [
        CausalTensorError::ShapeMismatch,
        CausalTensorError::DimensionMismatch,
        CausalTensorError::AxisOutOfBounds,
        CausalTensorError::EmptyTensor,
        CausalTensorError::InvalidOperation,
        CausalTensorError::UnorderableValue,
    ];

    for err in &errors {
        assert!(err.source().is_none());
    }
}
