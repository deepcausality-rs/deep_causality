/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, ManifoldTopology, PointCloud};

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
fn test_is_oriented() {
    let manifold = setup_triangle_manifold();
    // CheckManifold enforces orientation, so this must be true for a valid Manifold.
    assert!(manifold.is_oriented());
}

#[test]
fn test_satisfies_link_condition() {
    let manifold = setup_triangle_manifold();
    // CheckManifold enforces link condition, so this must be true.
    assert!(manifold.satisfies_link_condition());
}

#[test]
fn test_euler_characteristic() {
    let manifold = setup_triangle_manifold();
    // Euler characteristic = V - E + F
    // V = 3, E = 3, F = 1
    // X = 3 - 3 + 1 = 1
    assert_eq!(manifold.euler_characteristic(), 1);
}

#[test]
fn test_has_boundary() {
    let manifold = setup_triangle_manifold();
    // A single triangle has a boundary (its edges).
    // This assumes `has_boundary` checks if the manifold boundary is non-empty.
    assert!(manifold.has_boundary());
}
