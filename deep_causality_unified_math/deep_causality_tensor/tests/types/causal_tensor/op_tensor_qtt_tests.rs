/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{CausalTensor, CausalTensorError};

#[test]
fn test_quantize_then_merge_roundtrip() {
    // A [8, 3] tensor: quantize axis 0 (8 = 2^3) into three binary axes, then merge back.
    let data: Vec<f64> = (0..24).map(|i| i as f64).collect();
    let t = CausalTensor::new(data.clone(), vec![8, 3]).unwrap();

    let q = t.quantize_axis(0, 3).unwrap();
    assert_eq!(q.shape(), &[2, 2, 2, 3]);
    // Reshape is metadata-only: data order is preserved.
    assert_eq!(q.as_slice(), data.as_slice());

    let back = q.merge_binary_axes(0, 3).unwrap();
    assert_eq!(back.shape(), &[8, 3]);
    assert_eq!(back.as_slice(), data.as_slice());
}

#[test]
fn test_quantize_interior_axis() {
    let data: Vec<f64> = (0..12).map(|i| i as f64).collect();
    let t = CausalTensor::new(data, vec![3, 4]).unwrap();
    let q = t.quantize_axis(1, 2).unwrap(); // 4 = 2^2
    assert_eq!(q.shape(), &[3, 2, 2]);
    let back = q.merge_binary_axes(1, 2).unwrap();
    assert_eq!(back.shape(), &[3, 4]);
}

#[test]
fn test_quantize_errors() {
    let t = CausalTensor::new(vec![0.0; 6], vec![6]).unwrap();
    // 6 is not a power of two.
    assert!(matches!(
        t.quantize_axis(0, 3),
        Err(CausalTensorError::InvalidParameter(_))
    ));
    // axis out of bounds.
    assert!(matches!(
        t.quantize_axis(2, 1),
        Err(CausalTensorError::AxisOutOfBounds)
    ));
    // levels == 0.
    assert!(matches!(
        t.quantize_axis(0, 0),
        Err(CausalTensorError::InvalidParameter(_))
    ));
}

#[test]
fn test_merge_errors() {
    let t = CausalTensor::new(vec![0.0; 12], vec![2, 2, 3]).unwrap();
    // Third axis is not length 2.
    assert!(matches!(
        t.merge_binary_axes(1, 2),
        Err(CausalTensorError::InvalidParameter(_))
    ));
    // Block runs off the end.
    assert!(matches!(
        t.merge_binary_axes(2, 2),
        Err(CausalTensorError::AxisOutOfBounds)
    ));
}
