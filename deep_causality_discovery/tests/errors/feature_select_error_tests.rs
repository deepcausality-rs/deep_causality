/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::mrmr::MrmrError;
use deep_causality_discovery::FeatureSelectError;
use deep_causality_tensor::CausalTensorError;
use std::error::Error;

#[test]
fn test_display() {
    let mrmr_err = MrmrError::InvalidInput("test message".to_string());
    let err = FeatureSelectError::MrmrError(mrmr_err);
    assert_eq!(
        err.to_string(),
        "mRMR algorithm error: Invalid input: test message"
    );

    let tensor_err = CausalTensorError::DimensionMismatch;
    let err = FeatureSelectError::TensorError(tensor_err);
    assert!(
        err.to_string()
            .contains("Tensor error: CausalTensorError: Dimension mismatch error")
    );
}

#[test]
fn test_source() {
    let err = FeatureSelectError::TooFewFeatures(10, 5);
    assert!(err.source().is_none());

    let mrmr_err = MrmrError::InvalidInput("test message".to_string());
    let err = FeatureSelectError::MrmrError(mrmr_err);
    // MrmrError does not implement Error, so source() should be None
    assert!(err.source().is_none());

    let tensor_err = CausalTensorError::DimensionMismatch;
    let err = FeatureSelectError::TensorError(tensor_err);
    assert!(err.source().is_some());
    assert_eq!(
        err.source().unwrap().to_string(),
        "CausalTensorError: Dimension mismatch error"
    );
}

#[test]
fn test_from_mrmr_error() {
    let mrmr_err = MrmrError::InvalidInput("test message".to_string());
    let err = FeatureSelectError::from(mrmr_err);
    if let FeatureSelectError::MrmrError(_) = err {
        // Test passed
    } else {
        panic!("Incorrect error variant for MrmrError");
    }
}

#[test]
fn test_from_causal_tensor_error() {
    let tensor_err = CausalTensorError::DimensionMismatch;
    let err = FeatureSelectError::from(tensor_err);
    if let FeatureSelectError::TensorError(_) = err {
        // Test passed
    } else {
        panic!("Incorrect error variant for CausalTensorError");
    }
}
