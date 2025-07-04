/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::*;

fn get_dynamic_graph() -> UltraGraphContainer<String, u32> {
    let mut g = UltraGraphWeighted::new();
    g.add_node("A".to_string()).unwrap(); // 0
    g.add_node("B".to_string()).unwrap(); // 1
    g.add_node("C".to_string()).unwrap(); // 2
    g.add_edge(0, 1, 10).unwrap();
    g.add_edge(1, 2, 20).unwrap();
    g.add_root_node("ROOT".to_string()).unwrap(); // 3
    g
}

fn get_static_graph() -> UltraGraphContainer<String, u32> {
    let mut g = get_dynamic_graph();
    g.freeze();
    g
}

// Test functions for both dynamic and static graphs
fn test_view_on_graph<F>(graph_provider: F)
where
    F: Fn() -> UltraGraphContainer<String, u32>,
{
    let g = graph_provider();

    // is_empty
    assert!(!g.is_empty());

    // contains_node
    assert!(g.contains_node(0));
    assert!(g.contains_node(3));
    assert!(!g.contains_node(4));

    // get_node
    assert_eq!(g.get_node(0), Some(&"A".to_string()));
    assert_eq!(g.get_node(4), None);

    // number_nodes
    assert_eq!(g.number_nodes(), 4);

    // contains_edge
    assert!(g.contains_edge(0, 1));
    assert!(!g.contains_edge(1, 0));

    // number_edges
    assert_eq!(g.number_edges(), 2);

    // get_edges
    let edges = g.get_edges(1).unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0], (2, &20));
    assert!(g.get_edges(0).is_some());
    assert!(g.get_edges(2).unwrap().is_empty()); // No outgoing edges from C
    assert!(g.get_edges(99).is_none());

    // contains_root_node
    assert!(g.contains_root_node());

    // get_root_node
    assert_eq!(g.get_root_node(), Some(&"ROOT".to_string()));

    // get_root_index
    assert_eq!(g.get_root_index(), Some(3));
}

#[test]
fn test_view_on_dynamic_graph() {
    test_view_on_graph(get_dynamic_graph);
}

#[test]
fn test_view_on_static_graph() {
    test_view_on_graph(get_static_graph);
}

#[test]
fn test_is_frozen() {
    let dynamic_g = get_dynamic_graph();
    assert!(!dynamic_g.is_frozen());

    let static_g = get_static_graph();
    assert!(static_g.is_frozen());
}

#[test]
fn test_empty_graph_view() {
    let g: UltraGraph<i32> = UltraGraph::new();
    assert!(g.is_empty());
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
    assert!(!g.contains_node(0));
    assert!(!g.contains_edge(0, 0));
    assert!(!g.contains_root_node());
    assert_eq!(g.get_root_node(), None);
    assert_eq!(g.get_root_index(), None);
}

#[test]
fn test_graph_without_root() {
    let mut g = UltraGraph::new();
    g.add_node(1).unwrap();
    assert!(!g.contains_root_node());
    assert_eq!(g.get_root_node(), None);
    assert_eq!(g.get_root_index(), None);

    g.freeze();
    assert!(!g.contains_root_node());
    assert_eq!(g.get_root_node(), None);
    assert_eq!(g.get_root_index(), None);
}
