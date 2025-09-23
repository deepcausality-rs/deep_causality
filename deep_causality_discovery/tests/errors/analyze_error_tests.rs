/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::AnalyzeError;
use deep_causality_tensor::CausalTensorError;
use std::error::Error;

#[test]
fn test_display() {
    let err = AnalyzeError::EmptyResult;
    assert_eq!(err.to_string(), "The causal discovery result is empty.");

    let err = AnalyzeError::AnalysisFailed("Test failure".to_string());
    assert_eq!(err.to_string(), "Analysis failed: Test failure");

    let tensor_err = CausalTensorError::DimensionMismatch;
    let err = AnalyzeError::TensorError(tensor_err);
    assert!(err.to_string().contains("Tensor error during analysis"));
}

#[test]
fn test_source() {
    let err = AnalyzeError::EmptyResult;
    assert!(err.source().is_none());

    let err = AnalyzeError::AnalysisFailed("Test failure".to_string());
    assert!(err.source().is_none());

    let tensor_err = CausalTensorError::DimensionMismatch;
    let err = AnalyzeError::TensorError(tensor_err);
    assert!(err.source().is_some());
}

#[test]
fn test_from_causal_tensor_error() {
    let tensor_err = CausalTensorError::DimensionMismatch;
    let err = AnalyzeError::from(tensor_err);
    if let AnalyzeError::TensorError(_) = err {
        // Test passed
    } else {
        panic!("Incorrect error variant");
    }
}
