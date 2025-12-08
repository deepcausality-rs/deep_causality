/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry};

// Helper to create a manifold with a known metric i.e. Regge geometry
fn setup_manifold_with_metric() -> Manifold<f64> {
    // Create a single triangle (0-1-2) with known edge lengths.
    // Let's use a 3-4-5 right triangle for easy area calculation.
    // Lengths: 0-1 = 3, 0-2 = 4, 1-2 = 5 (hypotenuse)

    let points = CausalTensor::new(vec![0.0, 0.0, 3.0, 0.0, 0.0, 4.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let point_cloud = PointCloud::new(points, metadata, 0).unwrap();

    let complex = point_cloud.triangulate(6.0).unwrap();

    // We need to manually build the metric (edge lengths).
    // The metric must be a CausalTensor<f64> (1D array of edge lengths).
    // The order corresponds to the order of simplices in the 1-skeleton.

    let skeleton1 = complex.skeletons().get(1).unwrap();
    let num_edges = skeleton1.simplices().len();
    let mut edge_lengths_vec = Vec::with_capacity(num_edges);

    for simplex in skeleton1.simplices() {
        let u = simplex.vertices()[0];
        let v = simplex.vertices()[1];

        // Determine length based on vertices
        let len = if (u == 0 && v == 1) || (u == 1 && v == 0) {
            3.0
        } else if (u == 0 && v == 2) || (u == 2 && v == 0) {
            4.0
        } else if (u == 1 && v == 2) || (u == 2 && v == 1) {
            5.0
        } else {
            1.0 // Should not happen in a single triangle
        };

        edge_lengths_vec.push(len);
    }

    let edge_lengths_tensor = CausalTensor::new(edge_lengths_vec, vec![num_edges]).unwrap();
    let regge = ReggeGeometry::new(edge_lengths_tensor);

    let data_len = complex.total_simplices();
    let data = CausalTensor::new(vec![0.0; data_len], vec![data_len]).unwrap();

    Manifold::with_metric(complex, data, Some(regge), 0).unwrap()
}

#[test]
fn test_simplex_volume_squared_0d() {
    let manifold = setup_manifold_with_metric();
    // Get a vertex (0-simplex)
    let s0 = manifold.complex().skeletons()[0].simplices()[0].clone();

    // Volume squared of a point is 1.0
    let vol_sq = manifold.simplex_volume_squared(&s0).unwrap();
    assert!((vol_sq - 1.0).abs() < 1e-9);
}

#[test]
fn test_simplex_volume_squared_1d() {
    let manifold = setup_manifold_with_metric();
    // Get edge (0,1) which has length 3. Squared = 9.
    let s1 = manifold.complex().skeletons()[1]
        .simplices()
        .iter()
        .find(|s| s.vertices().contains(&0) && s.vertices().contains(&1))
        .unwrap()
        .clone();

    let vol_sq = manifold.simplex_volume_squared(&s1).unwrap();
    assert!((vol_sq - 9.0).abs() < 1e-9);

    // Edge (0,2) length 4. Squared = 16.
    let s2 = manifold.complex().skeletons()[1]
        .simplices()
        .iter()
        .find(|s| s.vertices().contains(&0) && s.vertices().contains(&2))
        .unwrap()
        .clone();
    let vol_sq2 = manifold.simplex_volume_squared(&s2).unwrap();
    assert!((vol_sq2 - 16.0).abs() < 1e-9);
}

#[test]
fn test_simplex_volume_squared_2d() {
    let manifold = setup_manifold_with_metric();
    // Triangle (0,1,2). Area = 0.5 * 3 * 4 = 6.
    // Squared Area = 36.
    let s2 = manifold.complex().skeletons()[2].simplices()[0].clone();

    let vol_sq = manifold.simplex_volume_squared(&s2).unwrap();
    assert!(
        (vol_sq - 36.0).abs() < 1e-4,
        "Expected 36.0, got {}",
        vol_sq
    );
}
