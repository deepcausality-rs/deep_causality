/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for Manifold covariance analysis (covariance_cpu.rs)

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

/// Helper to create a valid manifold with specified data values
fn setup_manifold_with_data(data_values: Vec<f64>) -> Manifold<f64, f64> {
    // Create a simple triangle from points
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let point_cloud = PointCloud::new(points, metadata, 0).unwrap();

    let complex = point_cloud.triangulate(1.2).unwrap();
    // Complex has 3 vertices, 3 edges, 1 face. Total 7 simplices.
    let data = CausalTensor::new(data_values, vec![7]).unwrap();

    Manifold::new(complex, data, 0).unwrap()
}

#[test]
fn test_covariance_matrix_cpu_success() {
    // Create manifold with varied data for meaningful covariance
    let manifold = setup_manifold_with_data(vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0]);

    let result = manifold.covariance_matrix();
    assert!(result.is_ok(), "Covariance computation should succeed");

    let cov_matrix = result.unwrap();
    assert_eq!(cov_matrix.len(), 1, "Should be 1x1 covariance matrix");
    assert_eq!(cov_matrix[0].len(), 1, "Should be 1x1 covariance matrix");

    // Variance should be positive for varied data
    assert!(cov_matrix[0][0] > 0.0, "Variance should be positive");
}

#[test]
fn test_covariance_matrix_cpu_uniform_data() {
    // Create manifold with uniform data - variance should be 0
    let manifold = setup_manifold_with_data(vec![5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0]);

    let result = manifold.covariance_matrix();
    assert!(result.is_ok(), "Covariance computation should succeed");

    let cov_matrix = result.unwrap();
    assert!(
        cov_matrix[0][0].abs() < 1e-10,
        "Variance of uniform data should be zero"
    );
}

#[test]
fn test_eigen_covariance_cpu_1x1() {
    // Create manifold with varied data
    let manifold = setup_manifold_with_data(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);

    let result = manifold.eigen_covariance();
    assert!(result.is_ok(), "Eigen covariance should succeed for 1x1");

    let eigenvalues = result.unwrap();
    assert_eq!(eigenvalues.len(), 1, "Should have one eigenvalue");

    // Eigenvalue equals variance for 1x1 matrix
    let cov = manifold.covariance_matrix().unwrap();
    assert!(
        (eigenvalues[0] - cov[0][0]).abs() < 1e-10,
        "Eigenvalue should equal variance"
    );
}
