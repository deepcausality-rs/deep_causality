/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVectorError, Metric};

#[test]
fn test_dimension_mismatch_display() {
    let err = CausalMultiVectorError::dimension_mismatch(4, 2);
    assert_eq!(
        format!("{}", err),
        "Dimension mismatch: expected 4, found 2"
    );
}

#[test]
fn test_data_length_mismatch_display() {
    let err = CausalMultiVectorError::data_length_mismatch(16, 8);
    assert_eq!(
        format!("{}", err),
        "Data length mismatch: expected 16, found 8"
    );
}

#[test]
fn test_zero_magnitude_display() {
    let err = CausalMultiVectorError::zero_magnitude();
    assert_eq!(
        format!("{}", err),
        "Operation requires non-zero magnitude (e.g., inverse of zero)"
    );
}

#[test]
fn test_metric_mismatch_display() {
    let m1 = Metric::Euclidean(2);
    let m2 = Metric::Minkowski(2);
    let err = CausalMultiVectorError::metric_mismatch(m1, m2);
    assert_eq!(
        format!("{}", err),
        "Metric mismatch between operands: Euclidean(2) vs Minkowski(2)"
    );
}

#[test]
fn test_debug_impl() {
    let err = CausalMultiVectorError::zero_magnitude();
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("CausalMultiVectorError"));
    assert!(debug_str.contains("ZeroMagnitude"));
}
