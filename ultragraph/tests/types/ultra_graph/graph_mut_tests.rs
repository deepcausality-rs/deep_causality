/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::*;

#[test]
fn test_add_node() {
    let mut g = UltraGraph::new();
    let id1 = g.add_node(10).unwrap();
    let id2 = g.add_node(20).unwrap();
    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
    assert_eq!(g.number_nodes(), 2);
    assert_eq!(*g.get_node(id1).unwrap(), 10);
}

#[test]
fn test_add_node_to_frozen_graph() {
    let mut g = UltraGraph::new();
    g.add_node(1).unwrap();
    g.freeze();
    assert!(matches!(g.add_node(2), Err(GraphError::GraphIsFrozen)));
}

#[test]
fn test_update_node() {
    let mut g = UltraGraph::new();
    let id = g.add_node(10).unwrap();
    g.update_node(id, 100).unwrap();
    assert_eq!(*g.get_node(id).unwrap(), 100);
}

#[test]
fn test_update_node_on_frozen_graph() {
    let mut g = UltraGraph::new();
    let id = g.add_node(1).unwrap();
    g.freeze();
    assert!(matches!(
        g.update_node(id, 2),
        Err(GraphError::GraphIsFrozen)
    ));
}

#[test]
fn test_update_non_existent_node() {
    let mut g: UltraGraph<i32> = UltraGraph::new();
    assert!(g.update_node(0, 100).is_err());
}

#[test]
fn test_remove_node() {
    let mut g = UltraGraph::new();
    let id0 = g.add_node(10).unwrap();
    let id1 = g.add_node(20).unwrap();
    g.add_edge(id0, id1, ()).unwrap();
    assert_eq!(g.number_nodes(), 2);
    assert_eq!(g.number_edges(), 1);

    g.remove_node(id0).unwrap();
    assert_eq!(g.number_nodes(), 1);
    assert!(!g.contains_node(id0));
    assert_eq!(g.number_edges(), 0); // Edge should be removed
}

#[test]
fn test_remove_node_from_frozen_graph() {
    let mut g = UltraGraph::new();
    let id = g.add_node(1).unwrap();
    g.freeze();
    assert!(matches!(g.remove_node(id), Err(GraphError::GraphIsFrozen)));
}

#[test]
fn test_remove_non_existent_node() {
    let mut g: UltraGraph<i32> = UltraGraph::new();
    assert!(g.remove_node(0).is_err());
}

#[test]
fn test_add_edge() {
    let mut g = UltraGraph::new();
    let id0 = g.add_node(10).unwrap();
    let id1 = g.add_node(20).unwrap();
    g.add_edge(id0, id1, ()).unwrap();
    assert_eq!(g.number_edges(), 1);
    assert!(g.contains_edge(id0, id1));
}

#[test]
fn test_add_edge_to_frozen_graph() {
    let mut g = UltraGraph::new();
    let id0 = g.add_node(10).unwrap();
    let id1 = g.add_node(20).unwrap();
    g.freeze();
    assert!(matches!(
        g.add_edge(id0, id1, ()),
        Err(GraphError::GraphIsFrozen)
    ));
}

#[test]
fn test_add_edge_invalid_node() {
    let mut g = UltraGraph::new();
    let id0 = g.add_node(10).unwrap();
    assert!(g.add_edge(id0, 99, ()).is_err());
    assert!(g.add_edge(99, id0, ()).is_err());
}

#[test]
fn test_remove_edge() {
    let mut g = UltraGraph::new();
    let id0 = g.add_node(10).unwrap();
    let id1 = g.add_node(20).unwrap();
    g.add_edge(id0, id1, ()).unwrap();
    assert!(g.contains_edge(id0, id1));

    g.remove_edge(id0, id1).unwrap();
    assert!(!g.contains_edge(id0, id1));
    assert_eq!(g.number_edges(), 0);
}

#[test]
fn test_remove_edge_from_frozen_graph() {
    let mut g = UltraGraph::new();
    let id0 = g.add_node(10).unwrap();
    let id1 = g.add_node(20).unwrap();
    g.add_edge(id0, id1, ()).unwrap();
    g.freeze();
    assert!(matches!(
        g.remove_edge(id0, id1),
        Err(GraphError::GraphIsFrozen)
    ));
}

#[test]
fn test_remove_non_existent_edge() {
    let mut g = UltraGraph::new();
    let id0 = g.add_node(10).unwrap();
    let id1 = g.add_node(20).unwrap();
    assert!(g.remove_edge(id0, id1).is_err());
}

#[test]
fn test_add_root_node() {
    let mut g = UltraGraph::new();
    let id = g.add_root_node(100).unwrap();
    assert!(g.contains_root_node());
    assert_eq!(g.get_root_index(), Some(id));
    assert_eq!(*g.get_root_node().unwrap(), 100);
}

#[test]
fn test_add_root_node_to_frozen_graph() {
    let mut g = UltraGraph::new();
    g.freeze();
    assert!(matches!(
        g.add_root_node(100),
        Err(GraphError::GraphIsFrozen)
    ));
}

#[test]
fn test_clear() {
    let mut g = UltraGraph::new();
    g.add_node(1).unwrap();
    g.add_edge(0, 0, ()).unwrap();
    g.add_root_node(100).unwrap();
    g.clear().unwrap();
    assert!(g.is_empty());
    assert!(!g.contains_root_node());
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
    assert!(!g.is_frozen());
}

#[test]
fn test_clear_frozen_graph() {
    let mut g = UltraGraph::new();
    g.add_node(1).unwrap();
    g.freeze();
    assert!(g.is_frozen());
    g.clear().unwrap();
    assert!(g.is_empty());
    assert!(!g.is_frozen());
}
