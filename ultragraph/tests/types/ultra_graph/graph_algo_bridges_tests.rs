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
fn tree_edges_are_all_bridges() {
    let g = build(&[(0, 1), (1, 2), (1, 3)], 4);
    assert_eq!(g.bridges().unwrap(), vec![(0, 1), (1, 2), (1, 3)]);
}

#[test]
fn cycle_has_no_bridges() {
    let g = build(&[(0, 1), (1, 2), (2, 0)], 3);
    assert!(g.bridges().unwrap().is_empty());
}

#[test]
fn two_cycles_joined_by_single_edge() {
    // Cycle {0,1,2} and {3,4,5} joined by edge (2,3).
    let g = build(&[(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 5), (5, 3)], 6);
    assert_eq!(g.bridges().unwrap(), vec![(2, 3)]);
}

#[test]
fn empty_graph_returns_empty() {
    let g = build(&[], 0);
    assert!(g.bridges().unwrap().is_empty());
}

#[test]
fn edgeless_graph_returns_empty() {
    let g = build(&[], 5);
    assert!(g.bridges().unwrap().is_empty());
}

#[test]
fn complete_graph_k4_has_no_bridges() {
    let g = build(&[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)], 4);
    assert!(g.bridges().unwrap().is_empty());
}

#[test]
fn star_graph_every_edge_is_bridge() {
    let g = build(&[(0, 1), (0, 2), (0, 3), (0, 4)], 5);
    assert_eq!(g.bridges().unwrap(), vec![(0, 1), (0, 2), (0, 3), (0, 4)]);
}

#[test]
fn parallel_directed_edges_canonicalized_to_min_max() {
    // Both (1,0) and (0,1) stored — should report (0,1) exactly once.
    let g = build(&[(1, 0), (0, 1)], 2);
    assert_eq!(g.bridges().unwrap(), vec![(0, 1)]);
}

#[test]
fn parallel_multi_edges_are_not_bridges() {
    // Two parallel directed edges (0,1) stored twice in the same direction.
    // In the undirected view this is a multigraph with multiplicity 2;
    // removing one parallel edge leaves the other, so neither is a bridge.
    let g = build(&[(0, 1), (0, 1)], 2);
    assert!(g.bridges().unwrap().is_empty());
}

#[test]
fn bridge_plus_parallel_multi_edge_isolates_correctly() {
    // {0,1} is a 2-multi-edge (not a bridge); (1,2) is a single edge (bridge).
    let g = build(&[(0, 1), (0, 1), (1, 2)], 3);
    assert_eq!(g.bridges().unwrap(), vec![(1, 2)]);
}

#[test]
fn self_loops_do_not_create_bridges() {
    // Triangle plus self-loops — still no bridges.
    let g = build(&[(0, 1), (1, 2), (2, 0), (0, 0), (1, 1)], 3);
    assert!(g.bridges().unwrap().is_empty());
}

#[test]
fn disconnected_graph_per_component() {
    // Cycle {0,1,2} ; edge {3,4}.
    let g = build(&[(0, 1), (1, 2), (2, 0), (3, 4)], 5);
    assert_eq!(g.bridges().unwrap(), vec![(3, 4)]);
}

#[test]
fn dynamic_graph_errors_with_graph_not_frozen() {
    let mut g = UltraGraphWeighted::<i32, ()>::new();
    g.add_node(0).unwrap();
    match g.bridges() {
        Err(GraphError::GraphNotFrozen) => {}
        other => panic!("expected GraphNotFrozen, got {:?}", other),
    }
}
