/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::klein_gordon;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry, SimplicialManifold};

// Helper to create a simple triangular manifold with a unit-edge metric
// attached (required by R4.5: `klein_gordon` wrapper calls
// `manifold.laplacian()`).
fn create_simple_manifold() -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(
        vec![
            0.0, 0.0, // v0
            1.0, 0.0, // v1
            0.5, 0.866, // v2
        ],
        vec![3, 2],
    )
    .unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num_simplices = complex.total_simplices();
    let num_edges = complex.skeletons()[1].simplices().len();
    let initial_data = vec![1.0; num_simplices];
    let metric =
        ReggeGeometry::new(CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap());
    Manifold::with_metric(
        complex,
        CausalTensor::new(initial_data, vec![num_simplices]).unwrap(),
        Some(metric),
        0,
    )
    .unwrap()
}

#[test]
fn test_klein_gordon_wrapper_success() {
    let manifold = create_simple_manifold();
    // Use mass = 0.0 for simple free field case or any value
    let effect = klein_gordon(&manifold, 0.5);
    // Success if manifold data shape matches what kernel expects (it does for simple manifold)
    assert!(effect.is_ok());
}

#[test]
fn test_klein_gordon_wrapper_error() {
    let manifold = create_simple_manifold();
    let effect = klein_gordon(&manifold, 1e200); // Massive mass
    // It should return Err because m^2 overflows and we check for finiteness.
    assert!(effect.is_err());
}
