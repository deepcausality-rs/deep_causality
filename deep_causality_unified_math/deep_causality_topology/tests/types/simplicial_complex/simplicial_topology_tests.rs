/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::utils_tests::create_triangle_complex;
use deep_causality_topology::{Simplex, SimplicialComplexBuilder, SimplicialTopology};

#[test]
fn test_simplicial_complex_new() {
    let complex = create_triangle_complex();
    assert_eq!(complex.skeletons().len(), 3); // 0, 1, 2-skeletons
    assert_eq!(complex.boundary_operators().len(), 2); // B1, B2
    assert!(complex.coboundary_operators().is_empty()); // Not implemented in helper
}

#[test]
fn test_simplicial_topology_max_dimension() {
    let complex = create_triangle_complex();
    assert_eq!(complex.max_simplex_dimension(), 2);
}

#[test]
fn test_simplicial_topology_num_simplices_at_grade() {
    let complex = create_triangle_complex();

    assert_eq!(complex.num_simplices_at_grade(0).unwrap(), 3);
    assert_eq!(complex.num_simplices_at_grade(1).unwrap(), 3);
    assert_eq!(complex.num_simplices_at_grade(2).unwrap(), 1);

    assert!(complex.num_simplices_at_grade(3).is_err());
}

#[test]
fn test_simplicial_topology_get_simplex() {
    let complex = create_triangle_complex();

    let s0 = complex.get_simplex(0, 0).unwrap();
    assert_eq!(s0.vertices().len(), 1); // 0-simplex has 1 vertex

    let s1 = complex.get_simplex(1, 0).unwrap();
    assert_eq!(s1.vertices().len(), 2); // 1-simplex has 2 vertices

    let s2 = complex.get_simplex(2, 0).unwrap();
    assert_eq!(s2.vertices().len(), 3); // 2-simplex has 3 vertices

    // Out of bounds index
    assert!(complex.get_simplex(0, 99).is_err());
    // Out of bounds grade
    assert!(complex.get_simplex(99, 0).is_err());
}

#[test]
fn test_simplicial_topology_contains_simplex() {
    let complex = create_triangle_complex();

    // Check known existing simplex
    let existing = complex.get_simplex(0, 0).unwrap();
    assert!(complex.contains_simplex(existing));

    // Check non-existing simplex (vertex 99)
    let non_existing = Simplex::new(vec![99]);
    assert!(!complex.contains_simplex(&non_existing));

    // Check non-existing grade (e.g. 4 vertices -> 3-simplex)
    // Triangle complex max is 2-simplex
    let invalid_dim = Simplex::new(vec![0, 1, 2, 3]);
    assert!(!complex.contains_simplex(&invalid_dim));

    // Empty simplex
    let empty = Simplex::new(vec![]);
    assert!(!complex.contains_simplex(&empty));
}

#[test]
fn test_skeletons_getter() {
    let mut builder = SimplicialComplexBuilder::new(1);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    builder.add_simplex(Simplex::new(vec![1])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1])).unwrap();
    let complex = builder.build::<f64>().unwrap();

    let skeletons = complex.skeletons();
    assert_eq!(skeletons.len(), 2); // 0-skeleton and 1-skeleton
}

#[test]
fn test_boundary_operators() {
    let mut builder = SimplicialComplexBuilder::new(1);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    builder.add_simplex(Simplex::new(vec![1])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1])).unwrap();
    let complex = builder.build::<f64>().unwrap();

    let boundary_ops = complex.boundary_operators();
    // Should have boundary operator for 1-skeleton
    assert!(!boundary_ops.is_empty());
}

#[test]
fn test_coboundary_operators() {
    let mut builder = SimplicialComplexBuilder::new(1);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    builder.add_simplex(Simplex::new(vec![1])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1])).unwrap();
    let complex = builder.build::<f64>().unwrap();

    let coboundary_ops = complex.coboundary_operators();
    // Should have coboundary operator
    assert!(!coboundary_ops.is_empty());
}

// =============================================================================
// Simplex vertices tests
// =============================================================================

#[test]
fn test_simplex_vertices() {
    let simplex = Simplex::new(vec![0, 1, 2]);
    let vertices = simplex.vertices();

    assert_eq!(vertices.len(), 3);
    assert!(vertices.contains(&0));
    assert!(vertices.contains(&1));
    assert!(vertices.contains(&2));
}

// =============================================================================
// Complex operations
// =============================================================================

#[test]
fn test_max_simplex_dimension() {
    let mut builder = SimplicialComplexBuilder::new(2);
    builder.add_simplex(Simplex::new(vec![0, 1, 2])).unwrap();
    let complex = builder.build::<f64>().unwrap();

    assert_eq!(complex.max_simplex_dimension(), 2);
}

#[test]
fn test_num_simplices_by_grade() {
    let mut builder = SimplicialComplexBuilder::new(1);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    builder.add_simplex(Simplex::new(vec![1])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1])).unwrap();
    let complex = builder.build::<f64>().unwrap();

    assert_eq!(complex.num_simplices_at_grade(0).unwrap(), 2);
    assert_eq!(complex.num_simplices_at_grade(1).unwrap(), 1);
}
