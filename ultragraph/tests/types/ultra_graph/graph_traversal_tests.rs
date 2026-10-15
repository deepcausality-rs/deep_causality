/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::*;

fn get_test_graph() -> UltraGraphContainer<i32, ()> {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap(); // 0
    g.add_node(1).unwrap(); // 1
    g.add_node(2).unwrap(); // 2
    g.add_node(3).unwrap(); // 3

    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(0, 2, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(2, 0, ()).unwrap(); // Cycle
    g.add_edge(2, 3, ()).unwrap();

    g.freeze();
    g
}

#[test]
fn test_outbound_edges_on_dynamic_graph() {
    let mut g = UltraGraph::new();
    let n0 = g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_edge(n0, 1, ()).unwrap();

    assert!(matches!(
        g.outbound_edges(n0),
        Err(GraphError::GraphNotFrozen)
    ));
}

#[test]
fn test_outbound_edges_on_static_graph() {
    let g = get_test_graph();
    let mut edges: Vec<_> = g.outbound_edges(0).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![1, 2]);

    let mut edges: Vec<_> = g.outbound_edges(1).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![2]);

    let mut edges: Vec<_> = g.outbound_edges(2).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![0, 3]);

    let edges: Vec<_> = g.outbound_edges(3).unwrap().collect();
    assert!(edges.is_empty());
}

#[test]
fn test_outbound_edges_invalid_node() {
    let g = get_test_graph();
    assert!(matches!(
        g.outbound_edges(99),
        Err(GraphError::NodeNotFound(99))
    ));
}

#[test]
fn test_inbound_edges_on_dynamic_graph() {
    let mut g = UltraGraph::new();
    let n0 = g.add_node(0).unwrap();
    let n1 = g.add_node(1).unwrap();
    g.add_edge(n0, n1, ()).unwrap();

    assert!(matches!(
        g.inbound_edges(n1),
        Err(GraphError::GraphNotFrozen)
    ));
}

#[test]
fn test_inbound_edges_on_static_graph() {
    let g = get_test_graph();

    let mut edges: Vec<_> = g.inbound_edges(0).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![2]);

    let mut edges: Vec<_> = g.inbound_edges(1).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![0]);

    let mut edges: Vec<_> = g.inbound_edges(2).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![0, 1]);

    let mut edges: Vec<_> = g.inbound_edges(3).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![2]);
}

#[test]
fn test_inbound_edges_invalid_node() {
    let g = get_test_graph();
    assert!(matches!(
        g.inbound_edges(99),
        Err(GraphError::NodeNotFound(99))
    ));
}

/// Spec `ultragraph-stable-node-ids`: traversal started at a tombstone returns `NodeNotFound`
/// with the requested position in both states, as it does for a position never issued.
#[test]
fn test_traversal_from_a_tombstone_fails_in_both_states() {
    let mut g = UltraGraph::new();
    for payload in 0..4 {
        g.add_node(payload).unwrap();
    }
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(0, 2, ()).unwrap();
    g.add_edge(2, 3, ()).unwrap();

    g.remove_node(1).unwrap();

    // Dynamic state: the tombstone, not the state, is what the error names.
    assert!(matches!(
        g.outbound_edges(1),
        Err(GraphError::NodeNotFound(1))
    ));
    assert!(matches!(
        g.inbound_edges(1),
        Err(GraphError::NodeNotFound(1))
    ));

    g.freeze();

    assert!(matches!(
        g.outbound_edges(1),
        Err(GraphError::NodeNotFound(1))
    ));
    assert!(matches!(
        g.inbound_edges(1),
        Err(GraphError::NodeNotFound(1))
    ));

    // The live nodes above the tombstone keep their positions and lose only the edges into it.
    let out: Vec<_> = g.outbound_edges(2).unwrap().collect();
    assert_eq!(out, vec![3]);
    let inbound: Vec<_> = g.inbound_edges(2).unwrap().collect();
    assert_eq!(inbound, vec![0]); // the edge 1 -> 2 left with the tombstone
}
