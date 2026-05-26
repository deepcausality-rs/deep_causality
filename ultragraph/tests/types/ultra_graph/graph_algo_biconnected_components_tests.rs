/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::HashSet;
use ultragraph::{GraphError, GraphMut, StructuralGraphAlgorithms, UltraGraphWeighted};

fn build(edges: &[(usize, usize)], n: usize) -> UltraGraphWeighted<i32, ()> {
    let mut g = UltraGraphWeighted::<i32, ()>::new();
    for i in 0..n {
        g.add_node(i as i32).unwrap();
    }
    for &(a, b) in edges {
        g.add_edge(a, b, ()).unwrap();
    }
    g.freeze();
    g
}

fn as_set(comps: Vec<Vec<usize>>) -> HashSet<Vec<usize>> {
    comps.into_iter().collect()
}

fn expect(comps: Vec<Vec<usize>>, expected: &[Vec<usize>]) {
    let got = as_set(comps);
    let want: HashSet<Vec<usize>> = expected.iter().cloned().collect();
    assert_eq!(got, want);
}

#[test]
fn cycle_is_single_component() {
    let g = build(&[(0, 1), (1, 2), (2, 0)], 3);
    expect(g.biconnected_components().unwrap(), &[vec![0, 1, 2]]);
}

#[test]
fn bow_tie_splits_at_cut_vertex() {
    let g = build(&[(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 2)], 5);
    expect(
        g.biconnected_components().unwrap(),
        &[vec![0, 1, 2], vec![2, 3, 4]],
    );
}

#[test]
fn bridge_between_two_cycles_produces_three_components() {
    let g = build(&[(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 5), (5, 3)], 6);
    expect(
        g.biconnected_components().unwrap(),
        &[vec![0, 1, 2], vec![2, 3], vec![3, 4, 5]],
    );
}

#[test]
fn tree_edges_each_form_a_two_vertex_component() {
    let g = build(&[(0, 1), (1, 2), (1, 3)], 4);
    expect(
        g.biconnected_components().unwrap(),
        &[vec![0, 1], vec![1, 2], vec![1, 3]],
    );
}

#[test]
fn empty_graph_returns_no_components() {
    let g = build(&[], 0);
    assert!(g.biconnected_components().unwrap().is_empty());
}

#[test]
fn isolated_only_returns_no_components() {
    let g = build(&[], 5);
    assert!(g.biconnected_components().unwrap().is_empty());
}

#[test]
fn complete_graph_k4_is_one_component() {
    let g = build(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)], 4);
    expect(g.biconnected_components().unwrap(), &[vec![0, 1, 2, 3]]);
}

#[test]
fn nested_cycles_share_articulation_vertex() {
    // Outer triangle 0-1-2 sharing edge with inner triangle 1-2-3.
    // Edges: (0,1),(1,2),(2,0),(1,3),(2,3) -> single biconnected component {0,1,2,3}.
    let g = build(&[(0, 1), (1, 2), (2, 0), (1, 3), (2, 3)], 4);
    expect(g.biconnected_components().unwrap(), &[vec![0, 1, 2, 3]]);
}

#[test]
fn isolated_vertices_are_excluded() {
    // Cycle {0,1,2} plus isolated 3.
    let g = build(&[(0, 1), (1, 2), (2, 0)], 4);
    let comps = g.biconnected_components().unwrap();
    expect(comps.clone(), &[vec![0, 1, 2]]);
    assert!(comps.iter().all(|c| !c.contains(&3)));
}

#[test]
fn self_loop_is_ignored() {
    let g = build(&[(0, 1), (1, 2), (2, 0), (0, 0)], 3);
    expect(g.biconnected_components().unwrap(), &[vec![0, 1, 2]]);
}

#[test]
fn disconnected_graph_emits_per_component() {
    // Cycle {0,1,2} ; edge {3,4}.
    let g = build(&[(0, 1), (1, 2), (2, 0), (3, 4)], 5);
    expect(
        g.biconnected_components().unwrap(),
        &[vec![0, 1, 2], vec![3, 4]],
    );
}

#[test]
fn dynamic_graph_errors_with_graph_not_frozen() {
    let mut g = UltraGraphWeighted::<i32, ()>::new();
    g.add_node(0).unwrap();
    match g.biconnected_components() {
        Err(GraphError::GraphNotFrozen) => {}
        other => panic!("expected GraphNotFrozen, got {:?}", other),
    }
}
