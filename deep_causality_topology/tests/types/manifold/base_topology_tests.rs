/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, Manifold, PointCloud};

// Setup function to create a manifold from a point cloud
// Helper adapted from differential_tests.rs
fn setup_triangle_manifold() -> Manifold<f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let point_cloud = PointCloud::new(points, metadata, 0).unwrap();

    let complex = point_cloud.triangulate(1.2).unwrap();
    // Complex has 3 vertices, 3 edges, 1 face. Total 7 simplices.
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0, 1.0, 2.0, 3.0, 100.0], vec![7]).unwrap();

    Manifold::new(complex, data, 0).unwrap()
}

#[test]
fn test_dimension() {
    let manifold = setup_triangle_manifold();
    // The triangle is a 2D simplex, so the manifold dimension is 2.
    assert_eq!(manifold.dimension(), 2);
}

#[test]
fn test_len() {
    let manifold = setup_triangle_manifold();
    // 3 vertices + 3 edges + 1 face = 7 simplices
    assert_eq!(manifold.len(), 7);
}

#[test]
fn test_num_elements_at_grade() {
    let manifold = setup_triangle_manifold();

    // Grade 0: 3 vertices
    assert_eq!(manifold.num_elements_at_grade(0), Some(3));

    // Grade 1: 3 edges
    assert_eq!(manifold.num_elements_at_grade(1), Some(3));

    // Grade 2: 1 face
    assert_eq!(manifold.num_elements_at_grade(2), Some(1));

    // Grade 3: None (doesn't exist)
    assert_eq!(manifold.num_elements_at_grade(3), None);
}
