/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::CausalDiscoveryError;
use deep_causality_tensor::CausalTensorError;
use std::error::Error;

#[test]
fn test_display() {
    let tensor_err = CausalTensorError::DimensionMismatch;
    let err = CausalDiscoveryError::TensorError(tensor_err);
    assert!(
        err.to_string()
            .contains("Tensor error during SURD: CausalTensorError: Dimension mismatch error")
    );
}

#[test]
fn test_source() {
    let tensor_err = CausalTensorError::DimensionMismatch;
    let err = CausalDiscoveryError::TensorError(tensor_err);
    assert!(err.source().is_some());
    assert_eq!(
        err.source().unwrap().to_string(),
        "CausalTensorError: Dimension mismatch error"
    );
}

#[test]
fn test_from_causal_tensor_error() {
    let tensor_err = CausalTensorError::DimensionMismatch;
    let CausalDiscoveryError::TensorError(inner_err) = CausalDiscoveryError::from(tensor_err);
    assert_eq!(inner_err, CausalTensorError::DimensionMismatch);
}
