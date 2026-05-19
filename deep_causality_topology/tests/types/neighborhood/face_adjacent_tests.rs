/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `FaceAdjacent` — chain-complex-generic top-cell face adjacency.

use deep_causality_topology::{
    FaceAdjacent, Neighborhood, Simplex, SimplicialComplex, SimplicialComplexBuilder,
};

fn two_triangles() -> SimplicialComplex<f64> {
    let mut b = SimplicialComplexBuilder::new(2);
    b.add_simplex(Simplex::new(vec![0, 1, 2])).unwrap();
    b.add_simplex(Simplex::new(vec![1, 2, 3])).unwrap();
    b.build::<f64>().unwrap()
}

#[test]
fn test_face_adjacent_two_triangles_share_edge() {
    let c = two_triangles();
    // Two top cells share edge {1,2} -> each is a neighbor of the other.
    let n0: Vec<_> = FaceAdjacent.neighbors(&c, 0).collect();
    let n1: Vec<_> = FaceAdjacent.neighbors(&c, 1).collect();
    assert_eq!(n0, vec![1]);
    assert_eq!(n1, vec![0]);
}

#[test]
fn test_face_adjacent_single_triangle_has_no_neighbors() {
    let mut b = SimplicialComplexBuilder::new(2);
    b.add_simplex(Simplex::new(vec![0, 1, 2])).unwrap();
    let c = b.build::<f64>().unwrap();
    let n: Vec<_> = FaceAdjacent.neighbors(&c, 0).collect();
    assert!(n.is_empty());
}

#[test]
fn test_face_adjacent_out_of_range_is_empty() {
    let c = two_triangles();
    let n: Vec<_> = FaceAdjacent.neighbors(&c, 999).collect();
    assert!(n.is_empty());
}

#[test]
fn test_face_adjacent_zero_max_dim_is_empty() {
    let c: SimplicialComplex<f64> =
        SimplicialComplex::new(Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let n: Vec<_> = FaceAdjacent.neighbors(&c, 0).collect();
    assert!(n.is_empty());
}
