/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    lorenz_gauge_kernel, magnetic_helicity_density_kernel, maxwell_gradient_kernel,
    poynting_vector_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

// Helper to create a simple triangular manifold
fn create_simple_manifold() -> Manifold<f64> {
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
    // Initialize with dummy data
    let initial_data = vec![1.0; num_simplices];
    Manifold::new(
        complex,
        CausalTensor::new(initial_data, vec![num_simplices]).unwrap(),
        0,
    )
    .unwrap()
}

#[test]
fn test_poynting_vector_kernel_valid() {
    // S = E x B
    // E = [0, 1, 0, 0] (x)
    // B = [0, 0, 1, 0] (y)
    // S should be x^y bivector
    let e = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = poynting_vector_kernel(&e, &b);
    assert!(result.is_ok());

    let s = result.unwrap();
    assert!(!s.data().is_empty());
}

#[test]
fn test_magnetic_helicity_density_kernel_valid() {
    // h = A . B
    let a = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = magnetic_helicity_density_kernel(&a, &b);
    assert!(result.is_ok());

    let h = result.unwrap();
    // Dot product of identical unit vectors is 1.0
    assert!((h - 1.0).abs() < 1e-10);
}

#[test]
fn test_maxwell_gradient_kernel_valid() {
    let manifold = create_simple_manifold();
    let result = maxwell_gradient_kernel(&manifold);
    assert!(result.is_ok());
    // F = dA. Just checking it computes without error on valid manifold
}

#[test]
fn test_lorenz_gauge_kernel_valid() {
    let manifold = create_simple_manifold();
    let result = lorenz_gauge_kernel(&manifold);
    assert!(result.is_ok());
}
