/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for Manifold constructors covering all error paths in constructors_cpu.rs

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry, TopologyErrorEnum};

/// Helper to create a valid manifold complex and data
fn setup_valid_manifold_parts() -> (
    deep_causality_topology::SimplicialComplex<f64>,
    CausalTensor<f64>,
) {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let point_cloud = PointCloud::new(points, metadata, 0).unwrap();

    let complex = point_cloud.triangulate(1.2).unwrap();
    // Complex has 3 vertices, 3 edges, 1 face. Total 7 simplices.
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0], vec![7]).unwrap();

    (complex, data)
}

// =============================================================================
// new() error paths
// =============================================================================

#[test]
fn test_new_success() {
    let (complex, data) = setup_valid_manifold_parts();
    let result = Manifold::new(complex, data, 0);
    assert!(result.is_ok(), "Valid manifold construction should succeed");
}

#[test]
fn test_new_data_size_mismatch() {
    let (complex, _) = setup_valid_manifold_parts();

    // Create data with wrong size (5 instead of 7)
    let wrong_data = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![5]).unwrap();

    let result = Manifold::new(complex, wrong_data, 0);

    assert!(result.is_err(), "Should fail on data size mismatch");
    let err = result.unwrap_err();
    match err.0 {
        TopologyErrorEnum::InvalidInput(ref msg) => {
            assert!(
                msg.contains("Data size"),
                "Error should mention data size: {}",
                msg
            );
        }
        ref e => panic!("Expected InvalidInput error, got {:?}", e),
    }
}

#[test]
fn test_new_cursor_out_of_bounds() {
    let (complex, data) = setup_valid_manifold_parts();

    // Cursor is 100, but data length is 7, so cursor >= data.len()
    let result = Manifold::new(complex, data, 100);

    assert!(result.is_err(), "Should fail on cursor out of bounds");
    let err = result.unwrap_err();
    match err.0 {
        TopologyErrorEnum::IndexOutOfBounds(ref msg) => {
            assert!(
                msg.contains("cursor"),
                "Error should mention cursor: {}",
                msg
            );
        }
        ref e => panic!("Expected IndexOutOfBounds error, got {:?}", e),
    }
}

// =============================================================================
// with_metric() error paths
// =============================================================================

#[test]
fn test_with_metric_success() {
    let (complex, data) = setup_valid_manifold_parts();

    // Create metric with correct number of edge lengths (3 edges)
    let edge_lengths = CausalTensor::new(vec![1.0, 1.1, 1.2], vec![3]).unwrap();
    let metric = ReggeGeometry::new(edge_lengths);

    let result = Manifold::with_metric(complex, data, Some(metric), 0);
    assert!(result.is_ok(), "Valid manifold with metric should succeed");
}

#[test]
fn test_with_metric_edge_length_mismatch() {
    let (complex, data) = setup_valid_manifold_parts();

    // Create metric with wrong number of edge lengths (2 instead of 3)
    let wrong_edge_lengths = CausalTensor::new(vec![1.0, 1.1], vec![2]).unwrap();
    let wrong_metric = ReggeGeometry::new(wrong_edge_lengths);

    let result = Manifold::with_metric(complex, data, Some(wrong_metric), 0);

    assert!(result.is_err(), "Should fail on edge length mismatch");
    let err = result.unwrap_err();
    match err.0 {
        TopologyErrorEnum::InvalidInput(ref msg) => {
            assert!(
                msg.contains("edge_lengths") || msg.contains("1-simplices"),
                "Error should mention edge lengths: {}",
                msg
            );
        }
        ref e => panic!("Expected InvalidInput error, got {:?}", e),
    }
}

#[test]
fn test_with_metric_none_success() {
    let (complex, data) = setup_valid_manifold_parts();

    // None metric should work fine
    let result = Manifold::with_metric(complex, data, None, 0);
    assert!(result.is_ok(), "Manifold with None metric should succeed");
}

// =============================================================================
// clone_shallow() tests
// =============================================================================

#[test]
fn test_clone_shallow() {
    let (complex, data) = setup_valid_manifold_parts();
    let manifold = Manifold::new(complex, data, 3).unwrap();

    let cloned = manifold.clone_shallow();

    // Shallow clone should preserve complex and data
    assert_eq!(
        manifold.complex(),
        cloned.complex(),
        "Complex should be cloned"
    );
    assert_eq!(
        manifold.data().len(),
        cloned.data().len(),
        "Data should be cloned"
    );
}

// =============================================================================
// Getters tests (for manifold/getters coverage)
// =============================================================================

#[test]
fn test_manifold_getters() {
    let (complex, data) = setup_valid_manifold_parts();
    let edge_lengths = CausalTensor::new(vec![1.0, 1.1, 1.2], vec![3]).unwrap();
    let metric = ReggeGeometry::new(edge_lengths);
    let manifold = Manifold::with_metric(complex, data, Some(metric), 2).unwrap();

    // Test complex() getter
    assert!(
        !manifold.complex().skeletons().is_empty(),
        "complex() should return the underlying complex"
    );

    // Test data() getter
    assert_eq!(manifold.data().len(), 7, "data() should return tensor data");

    // Test metric() getter
    assert!(
        manifold.metric().is_some(),
        "metric() should return the metric"
    );
}
