/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::DataCleaningError;
use deep_causality_tensor::CausalTensorError;
use std::error::Error;

#[test]
fn test_data_cleaning_error_display() {
    let err = DataCleaningError::TensorError(CausalTensorError::ShapeMismatch);
    assert_eq!(
        err.to_string(),
        "DataCleaningError: Tensor Error: CausalTensorError: Shape mismatch error"
    );
}

#[test]
fn test_data_cleaning_error_source() {
    let causal_tensor_error = CausalTensorError::InvalidOperation;
    let err = DataCleaningError::TensorError(causal_tensor_error);
    assert!(err.source().is_some());
    assert_eq!(
        err.source().unwrap().to_string(),
        "CausalTensorError: Invalid operation error"
    );
}

#[test]
fn test_data_cleaning_error_from_causal_tensor_error() {
    let causal_tensor_error = CausalTensorError::DimensionMismatch;
    let err = DataCleaningError::from(causal_tensor_error);
    let DataCleaningError::TensorError(e) = err;
    assert_eq!(e, CausalTensorError::DimensionMismatch);
}
