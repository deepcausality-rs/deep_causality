/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, PointCloud, TopologyError};

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
    let result = PointCloud::<f64>::new(points, metadata, 0);
    assert!(result.is_err());
    match result {
        Err(TopologyError::InvalidInput(msg)) => {
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
        Err(TopologyError::InvalidInput(msg)) => {
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
        Err(TopologyError::IndexOutOfBounds(msg)) => {
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

#[test]
fn test_point_cloud_base_topology() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 1.0, 2.0, 2.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    assert_eq!(pc.dimension(), 0);
    assert_eq!(pc.len(), 3);
    assert!(!pc.is_empty());
    assert_eq!(pc.num_elements_at_grade(0), Some(3));
    assert_eq!(pc.num_elements_at_grade(1), None);
}

#[test]
fn test_point_cloud_triangulate_success() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let complex = pc.triangulate(1.1); // Radius covers edges (0,1) and (0,2) but not (1,2)
    assert!(complex.is_ok());
    let sc = complex.unwrap();

    // Expect 3 vertices (0-simplices)
    assert_eq!(sc.skeletons()[0].simplices().len(), 3);
    // Expect 2 edges (1-simplices)
    assert_eq!(sc.skeletons()[1].simplices().len(), 2);
    // No 2-simplices (face) expected
    assert_eq!(sc.skeletons().len(), 2);
}
