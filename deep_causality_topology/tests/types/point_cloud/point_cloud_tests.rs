/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{PointCloud, TopologyError, TopologyErrorEnum};

/// Helper to create a simple point cloud
fn create_point_cloud() -> PointCloud<f64, f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    PointCloud::new(points, metadata, 0).unwrap()
}

#[test]
fn test_point_cloud_new_success() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 1.0], vec![2, 2]).unwrap();
    let metadata = CausalTensor::new(vec![10.0, 20.0], vec![2]).unwrap();
    let pc = PointCloud::new(points, metadata, 0);
    assert!(pc.is_ok());
    let p = pc.unwrap();
    assert_eq!(p.len(), 2);
    assert!(!p.is_empty());
}

#[test]
fn test_point_cloud_new_empty_points() {
    let points = CausalTensor::new(vec![], vec![0, 0]).unwrap();
    let metadata = CausalTensor::new(vec![], vec![0]).unwrap();
    let result = PointCloud::<f64, f64>::new(points, metadata, 0);
    assert!(result.is_err());
    match result {
        Err(TopologyError(TopologyErrorEnum::InvalidInput(msg))) => {
            assert!(msg.contains("PointCloud `points` cannot be empty"));
        }
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_point_cloud_new_metadata_mismatch() {
    let points = CausalTensor::new(vec![0.0, 0.0], vec![1, 2]).unwrap();
    let metadata = CausalTensor::new(vec![10.0, 20.0], vec![2]).unwrap(); // 2 metadata items for 1 point
    let result = PointCloud::new(points, metadata, 0);
    assert!(result.is_err());
    match result {
        Err(TopologyError(TopologyErrorEnum::InvalidInput(msg))) => {
            assert!(msg.contains("Number of points and metadata items must match"));
        }
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_point_cloud_new_cursor_out_of_bounds() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 1.0], vec![2, 2]).unwrap();
    let metadata = CausalTensor::new(vec![10.0, 20.0], vec![2]).unwrap();
    let result = PointCloud::new(points, metadata, 5); // Cursor out of bounds for 2 points
    assert!(result.is_err());
    match result {
        Err(TopologyError(TopologyErrorEnum::IndexOutOfBounds(msg))) => {
            assert!(msg.contains("cursor out of bounds"));
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
}

#[test]
fn test_point_cloud_getters() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 1.0], vec![2, 2]).unwrap();
    let metadata = CausalTensor::new(vec![10.0, 20.0], vec![2]).unwrap();
    let pc = PointCloud::new(points.clone(), metadata.clone(), 0).unwrap();

    assert_eq!(pc.points(), &points);
    assert_eq!(pc.metadata(), &metadata);
    assert_eq!(pc.cursor(), 0);
}

#[test]
fn test_point_cloud_clone_shallow() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 1.0], vec![2, 2]).unwrap();
    let metadata = CausalTensor::new(vec![10.0, 20.0], vec![2]).unwrap();
    let pc = PointCloud::new(points, metadata, 1).unwrap();
    let shallow_clone = pc.clone_shallow();

    assert_eq!(shallow_clone.points(), pc.points());
    assert_eq!(shallow_clone.metadata(), pc.metadata());
    assert_eq!(shallow_clone.cursor(), 0); // Cursor should be reset
    assert_ne!(pc.cursor(), shallow_clone.cursor());
}

// =============================================================================
// len and is_empty coverage
// =============================================================================

#[test]
fn test_point_cloud_len() {
    let pc = create_point_cloud();
    assert_eq!(pc.len(), 3, "Should have 3 points");
}

#[test]
fn test_point_cloud_is_empty_false() {
    let pc = create_point_cloud();
    assert!(!pc.is_empty(), "Non-empty point cloud should return false");
}

#[test]
fn test_point_cloud_cursor() {
    let pc = create_point_cloud();
    assert_eq!(pc.cursor(), 0);
}

// =============================================================================
// Display coverage
// =============================================================================

#[test]
fn test_point_cloud_display() {
    let pc = create_point_cloud();
    let display_str = format!("{}", pc);

    assert!(display_str.contains("PointCloud"));
    assert!(display_str.contains("3")); // number of points
}

// =============================================================================
// Triangulation edge cases
// =============================================================================

#[test]
fn test_triangulate_varying_radius() {
    let pc = create_point_cloud();

    // Small radius - may produce fewer simplices
    let complex_small = pc.triangulate(0.5);
    assert!(complex_small.is_ok());

    // Large radius - should connect all points
    let complex_large = pc.triangulate(10.0);
    assert!(complex_large.is_ok());
}

#[test]
fn test_triangulate_zero_radius() {
    let pc = create_point_cloud();

    // Zero radius should produce only vertices
    let result = pc.triangulate(0.0);
    assert!(result.is_ok());
    let complex = result.unwrap();
    // Should at least have vertices
    assert!(!complex.skeletons().is_empty());
}
