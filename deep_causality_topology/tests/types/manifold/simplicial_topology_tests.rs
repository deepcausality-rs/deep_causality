/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, Simplex, SimplicialTopology};

// Setup function to create a manifold from a point cloud
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
fn test_max_simplex_dimension() {
    let manifold = setup_triangle_manifold();
    // The highest dimension is 2 (the face).
    assert_eq!(manifold.max_simplex_dimension(), 2);
}

#[test]
fn test_num_simplices_at_grade() {
    let manifold = setup_triangle_manifold();

    // Grade 0: 3 vertices
    assert_eq!(manifold.num_simplices_at_grade(0).unwrap(), 3);

    // Grade 1: 3 edges
    assert_eq!(manifold.num_simplices_at_grade(1).unwrap(), 3);

    // Grade 2: 1 face
    assert_eq!(manifold.num_simplices_at_grade(2).unwrap(), 1);

    // Grade 3: Error
    assert!(manifold.num_simplices_at_grade(3).is_err());
}

#[test]
fn test_get_simplex() {
    let manifold = setup_triangle_manifold();

    // Get a vertex (0-simplex)
    let s0 = manifold.get_simplex(0, 0).unwrap();
    assert_eq!(s0.vertices().len() - 1, 0);

    // Get an edge (1-simplex)
    let s1 = manifold.get_simplex(1, 0).unwrap();
    assert_eq!(s1.vertices().len() - 1, 1);

    // Get a face (2-simplex)
    let s2 = manifold.get_simplex(2, 0).unwrap();
    assert_eq!(s2.vertices().len() - 1, 2);

    // Out of bounds
    assert!(manifold.get_simplex(0, 99).is_err());

    // Invalid grade
    assert!(manifold.get_simplex(5, 0).is_err());
}

#[test]
fn test_contains_simplex() {
    let manifold = setup_triangle_manifold();

    // Reconstruct a known simplex
    // The triangulation determines exact indices, but usually vertices are 0, 1, 2.
    // Let's get one from the manifold to be sure.
    let real_simplex = manifold.get_simplex(0, 0).unwrap();
    assert!(manifold.contains_simplex(real_simplex));

    // Construct a simplex that definitely exists (a vertex)
    // Vertices are labeled 0, 1, 2 probably.
    let v0 = Simplex::new(vec![0]);
    assert!(manifold.contains_simplex(&v0));

    // Construct a simplex that doesn't exist (invalid vertex)
    let v_bad = Simplex::new(vec![999]);
    assert!(!manifold.contains_simplex(&v_bad));
}
