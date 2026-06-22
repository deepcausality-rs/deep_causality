/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::PointCloud;

// When the `points` tensor is 1-dimensional, `shape().get(1)` is `None`, so the
// Display impl falls back to `unwrap_or(&0)` for the "Point Dimensions" line.
#[test]
fn test_point_cloud_display_one_dimensional_shape_falls_back_to_zero() {
    // Shape [2]: one axis only -> get(1) is None.
    let points = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let metadata = CausalTensor::new(vec![10.0, 20.0], vec![2]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let display_str = format!("{}", pc);

    assert!(display_str.contains("PointCloud:"));
    assert!(display_str.contains("Point Dimensions: 0"));
}
