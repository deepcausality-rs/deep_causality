/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `CofaceAdjacent` — chain-complex-generic coface adjacency.

use deep_causality_topology::{
    CofaceAdjacent, Neighborhood, Simplex, SimplicialComplex, SimplicialComplexBuilder,
};

fn triangle_complex() -> SimplicialComplex<f64> {
    // 2-simplex {0,1,2} -> 3 vertices, 3 edges, 1 face. Builder fills the
    // coboundary operators automatically.
    let mut b = SimplicialComplexBuilder::new(2);
    b.add_simplex(Simplex::new(vec![0, 1, 2])).unwrap();
    b.build::<f64>().unwrap()
}

fn two_triangles_complex() -> SimplicialComplex<f64> {
    // Two triangles sharing edge {1,2}: {0,1,2} and {1,2,3}.
    let mut b = SimplicialComplexBuilder::new(2);
    b.add_simplex(Simplex::new(vec![0, 1, 2])).unwrap();
    b.add_simplex(Simplex::new(vec![1, 2, 3])).unwrap();
    b.build::<f64>().unwrap()
}

#[test]
fn test_coface_adjacent_single_triangle_each_edge_has_one_coface() {
    let c = triangle_complex();
    // Three edges, all belong to the single triangle (cell_id 0).
    for edge_id in 0..3 {
        let n: Vec<_> = CofaceAdjacent.neighbors(&c, edge_id).collect();
        assert_eq!(
            n,
            vec![0],
            "edge {edge_id} should have triangle 0 as coface"
        );
    }
}

#[test]
fn test_coface_adjacent_shared_edge_has_two_cofaces() {
    let c = two_triangles_complex();
    let mut found_shared = false;
    // Find the shared edge {1,2} and verify it has two cofaces, while the
    // remaining edges each have exactly one.
    // 5 edges: {0,1},{0,2},{1,2},{1,3},{2,3}.
    for edge_id in 0..5 {
        let n: Vec<_> = CofaceAdjacent.neighbors(&c, edge_id).collect();
        if n.len() == 2 {
            found_shared = true;
            assert_eq!(n, vec![0, 1], "shared edge cofaces must be sorted+deduped");
        } else {
            assert_eq!(n.len(), 1, "non-shared edges have exactly one coface");
        }
    }
    assert!(found_shared, "expected one shared edge with two cofaces");
}

#[test]
fn test_coface_adjacent_out_of_range_cell_is_empty() {
    let c = triangle_complex();
    let n: Vec<_> = CofaceAdjacent.neighbors(&c, 999).collect();
    assert!(n.is_empty(), "out-of-range cell yields empty neighborhood");
}

#[test]
fn test_coface_adjacent_zero_max_dim_is_empty() {
    // No skeletons -> max_dim() == 0 -> early return.
    let c: SimplicialComplex<f64> =
        SimplicialComplex::new(Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let n: Vec<_> = CofaceAdjacent.neighbors(&c, 0).collect();
    assert!(n.is_empty());
}

#[test]
fn test_coface_adjacent_iterator_drains() {
    // Smoke-check `CofaceAdjacentIter::next` end-condition: after collecting all
    // items the iterator yields `None`.
    let c = triangle_complex();
    let mut it = CofaceAdjacent.neighbors(&c, 0);
    let _first = it.next();
    while it.next().is_some() {}
    assert!(it.next().is_none());
}
