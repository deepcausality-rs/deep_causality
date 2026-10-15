/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::{
    EdgeKey, GraphError, GraphMut, ParallelEdges, StructuralGraphAlgorithms, UltraGraphWeighted,
};

/// Nodes `0..n` with payload equal to position, the given edges, frozen.
fn build_with<W: Clone + Default + EdgeKey>(
    edges: impl IntoIterator<Item = (usize, usize, W)>,
    n: usize,
) -> UltraGraphWeighted<i32, W> {
    let mut g = UltraGraphWeighted::<i32, W>::new();
    for i in 0..n {
        g.add_node(i as i32).unwrap();
    }
    for (a, b, w) in edges {
        g.add_edge(a, b, w).unwrap();
    }
    g.freeze();
    g
}

fn build(edges: &[(usize, usize)], n: usize) -> UltraGraphWeighted<i32, ()> {
    build_with(edges.iter().map(|&(a, b)| (a, b, ())), n)
}

/// A two-variant payload that is its own key; with `W = ()` a repeated pair is one edge, so
/// parallel edges need a keyed payload.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Kind {
    #[default]
    A,
    B,
}

impl EdgeKey for Kind {
    type Key = Kind;

    fn edge_key(&self) -> Self::Key {
        *self
    }
}

fn build_keyed(edges: &[(usize, usize, Kind)], n: usize) -> UltraGraphWeighted<i32, Kind> {
    build_with(edges.iter().copied(), n)
}

/// A node test and an edge test as plain function pointers.
type TestPair = (fn(&i32) -> bool, fn(&Kind) -> bool);

/// A projection that passes every node and every edge.
fn pass_all() -> TestPair {
    (|_| true, |_| true)
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
    // Two parallel directed edges 0 -> 1 with the keys A and B. Under `Distinct` the undirected
    // view is a multigraph with multiplicity 2: removing one parallel edge leaves the other, so
    // neither is a bridge.
    let g = build_keyed(&[(0, 1, Kind::A), (0, 1, Kind::B)], 2);
    let distinct = g.project(pass_all(), ParallelEdges::Distinct).unwrap();
    assert!(distinct.bridges().unwrap().is_empty());

    // Under `Merge` the pair is one connection, and removing it disconnects 0 from 1.
    let merged = g.project(pass_all(), ParallelEdges::Merge).unwrap();
    assert_eq!(merged.bridges().unwrap(), vec![(0, 1)]);

    // The direct call runs on the simple projection, which is `Merge`.
    assert_eq!(g.bridges().unwrap(), vec![(0, 1)]);
}

#[test]
fn bridge_plus_parallel_multi_edge_isolates_correctly() {
    // {0,1} carries two keyed edges; (1,2) is a single edge.
    let g = build_keyed(&[(0, 1, Kind::A), (0, 1, Kind::B), (1, 2, Kind::A)], 3);

    // `Distinct`: {0,1} is a 2-multi-edge (not a bridge); (1,2) is a bridge.
    let distinct = g.project(pass_all(), ParallelEdges::Distinct).unwrap();
    assert_eq!(distinct.bridges().unwrap(), vec![(1, 2)]);

    // `Merge`: {0,1} is one connection, so both connections are bridges.
    let merged = g.project(pass_all(), ParallelEdges::Merge).unwrap();
    assert_eq!(merged.bridges().unwrap(), vec![(0, 1), (1, 2)]);

    // The direct call runs on the simple projection, which is `Merge`.
    assert_eq!(g.bridges().unwrap(), vec![(0, 1), (1, 2)]);
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
