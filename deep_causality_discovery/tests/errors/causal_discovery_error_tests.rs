/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::{BrcdError, BrcdErrorEnum};
use deep_causality_discovery::CausalDiscoveryError;
use deep_causality_tensor::CausalTensorError;
use std::error::Error;

#[test]
fn test_display_tensor() {
    let err = CausalDiscoveryError::TensorError(CausalTensorError::DimensionMismatch);
    assert!(err.to_string().contains("Tensor error during discovery"));
}

#[test]
fn test_display_brcd() {
    let err = CausalDiscoveryError::Brcd(BrcdError(BrcdErrorEnum::EmptyData));
    assert!(err.to_string().contains("BRCD error during discovery"));
}

#[test]
fn test_source() {
    let err = CausalDiscoveryError::TensorError(CausalTensorError::DimensionMismatch);
    assert!(err.source().is_some());
    let brcd = CausalDiscoveryError::Brcd(BrcdError(BrcdErrorEnum::EmptyData));
    assert!(brcd.source().is_some());
}

#[test]
fn test_from_causal_tensor_error() {
    match CausalDiscoveryError::from(CausalTensorError::DimensionMismatch) {
        CausalDiscoveryError::TensorError(e) => assert_eq!(e, CausalTensorError::DimensionMismatch),
        other => panic!("expected TensorError, got {other:?}"),
    }
}

#[test]
fn test_from_brcd_error() {
    match CausalDiscoveryError::from(BrcdError(BrcdErrorEnum::EmptyData)) {
        CausalDiscoveryError::Brcd(e) => assert_eq!(e, BrcdError(BrcdErrorEnum::EmptyData)),
        other => panic!("expected Brcd, got {other:?}"),
    }
}
