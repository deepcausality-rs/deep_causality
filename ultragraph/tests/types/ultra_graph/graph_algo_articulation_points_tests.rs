/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

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

#[test]
fn linear_chain_interior_vertices_are_articulation() {
    // 0 - 1 - 2 - 3 - 4 ; cut vertices are 1, 2, 3.
    let g = build(&[(0, 1), (1, 2), (2, 3), (3, 4)], 5);
    assert_eq!(g.articulation_points().unwrap(), vec![1, 2, 3]);
}

#[test]
fn cycle_has_no_articulation_points() {
    let g = build(&[(0, 1), (1, 2), (2, 0)], 3);
    assert!(g.articulation_points().unwrap().is_empty());
}

#[test]
fn two_cycles_joined_at_one_vertex() {
    // Cycle {0,1,2} and {2,3,4} sharing vertex 2.
    let g = build(&[(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 2)], 5);
    assert_eq!(g.articulation_points().unwrap(), vec![2]);
}

#[test]
fn empty_graph_returns_empty() {
    let g = build(&[], 0);
    assert!(g.articulation_points().unwrap().is_empty());
}

#[test]
fn isolated_vertices_only_returns_empty() {
    let g = build(&[], 5);
    assert!(g.articulation_points().unwrap().is_empty());
}

#[test]
fn complete_graph_k4_has_no_articulation_points() {
    let g = build(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)], 4);
    assert!(g.articulation_points().unwrap().is_empty());
}

#[test]
fn star_graph_center_is_articulation() {
    // Center = 0; leaves = 1..4.
    let g = build(&[(0, 1), (0, 2), (0, 3), (0, 4)], 5);
    assert_eq!(g.articulation_points().unwrap(), vec![0]);
}

#[test]
fn self_loops_alone_yield_no_articulation_points() {
    let mut g = UltraGraphWeighted::<i32, ()>::new();
    for i in 0..3 {
        g.add_node(i).unwrap();
    }
    g.add_edge(0, 0, ()).unwrap();
    g.add_edge(1, 1, ()).unwrap();
    g.freeze();
    assert!(g.articulation_points().unwrap().is_empty());
}

#[test]
fn self_loop_does_not_affect_result() {
    // Bow-tie + self-loop on vertex 2.
    let g = build(&[(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 2), (2, 2)], 5);
    assert_eq!(g.articulation_points().unwrap(), vec![2]);
}

#[test]
fn disconnected_components_handled_independently() {
    // Cycle {0,1,2} ; edge {3,4} ; isolated 5.
    let g = build(&[(0, 1), (1, 2), (2, 0), (3, 4)], 6);
    assert!(g.articulation_points().unwrap().is_empty());
}

#[test]
fn dynamic_graph_errors_with_graph_not_frozen() {
    let mut g = UltraGraphWeighted::<i32, ()>::new();
    g.add_node(0).unwrap();
    // not frozen
    match g.articulation_points() {
        Err(GraphError::GraphNotFrozen) => {}
        other => panic!("expected GraphNotFrozen, got {:?}", other),
    }
}
