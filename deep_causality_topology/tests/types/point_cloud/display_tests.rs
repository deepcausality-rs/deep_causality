/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::PointCloud;

#[test]
fn test_point_cloud_display() {
    let points = CausalTensor::new(vec![0.0, 0.0], vec![1, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let display_str = format!("{}", pc);

    assert!(display_str.contains("PointCloud:"));
    assert!(display_str.contains("Number of Points: 1"));
    assert!(display_str.contains("Point Dimensions: 2"));
    assert!(display_str.contains("Points Data:"));
    assert!(display_str.contains("Metadata Data:"));
}
