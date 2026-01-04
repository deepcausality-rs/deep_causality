/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, PointCloud};

#[test]
fn test_point_cloud_base_topology() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 1.0, 2.0, 2.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    // Dimension of a PointCloud is 0 (it's a 0-complex equivalent)
    assert_eq!(pc.dimension(), 0);

    // Len is number of points
    assert_eq!(pc.len(), 3);

    // Is empty check
    assert!(!pc.is_empty());

    // Num elements at grade
    assert_eq!(pc.num_elements_at_grade(0), Some(3));
    assert_eq!(pc.num_elements_at_grade(1), None);
}
