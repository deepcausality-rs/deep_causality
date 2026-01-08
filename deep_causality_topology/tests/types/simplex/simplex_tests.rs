/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::Simplex;

#[test]
fn test_simplex_new() {
    let vertices = vec![0, 1, 2];
    let simplex = Simplex::new(vertices.clone());
    assert_eq!(simplex.vertices(), &vertices);
}

#[test]
fn test_simplex_getters() {
    let vertices = vec![5, 8, 12];
    let simplex = Simplex::new(vertices.clone());
    assert_eq!(simplex.vertices(), &vertices);
}

#[test]
fn test_simplex_subsimplex() {
    let simplex = Simplex::new(vec![0, 1, 2, 3]);

    // Subsimplex (face) with range 0..2 (vertices 0, 1)
    let sub1 = simplex.subsimplex(0..2);
    assert_eq!(sub1.vertices(), &vec![0, 1]);

    // Subsimplex (edge) with range 1..3 (vertices 1, 2)
    let sub2 = simplex.subsimplex(1..3);
    assert_eq!(sub2.vertices(), &vec![1, 2]);

    // Subsimplex (vertex) with range 2..3 (vertex 2)
    let sub3 = simplex.subsimplex(2..3);
    assert_eq!(sub3.vertices(), &vec![2]);

    // Full simplex as subsimplex
    let full = simplex.subsimplex(0..4);
    assert_eq!(full.vertices(), &vec![0, 1, 2, 3]);

    // Empty subsimplex
    let empty = simplex.subsimplex(0..0);
    assert_eq!(empty.vertices(), &Vec::<usize>::new());
}

#[test]
fn test_simplex_display() {
    let simplex = Simplex::new(vec![10, 20, 30]);
    assert_eq!(format!("{}", simplex), "Simplex(10, 20, 30)");

    let vertex_simplex = Simplex::new(vec![5]);
    assert_eq!(format!("{}", vertex_simplex), "Simplex(5)");

    let empty_simplex = Simplex::new(vec![]);
    assert_eq!(format!("{}", empty_simplex), "Simplex()");
}

#[test]
fn test_simplex_equality() {
    let s1 = Simplex::new(vec![0, 1, 2]);
    let s2 = Simplex::new(vec![0, 1, 2]);
    let s3 = Simplex::new(vec![0, 2, 1]); // Different order, but should be canonicalized

    assert_eq!(s1, s2);
    assert_eq!(s1, s2);
    // Note: Simplex::new sorts vertices internally to ensure canonical representation.
    // Even if input is unsorted, equality holds.
    assert_eq!(s1, s3);
}

#[test]
fn test_simplex_order() {
    let s1 = Simplex::new(vec![0, 1]);
    let s2 = Simplex::new(vec![0, 1, 2]);
    let s3 = Simplex::new(vec![1, 2]);

    assert!(s1 < s2);
    assert!(s1 < s3);
    assert!(s2 < s3);
}
