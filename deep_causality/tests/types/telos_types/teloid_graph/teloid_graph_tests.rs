/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{DeonticError, TeloidGraph, TeloidID, Teloidable};

#[test]
fn test_teloid_graph_new() {
    let graph = TeloidGraph::new();
    assert!(graph.is_empty());
    assert_eq!(graph.number_nodes(), 0);
    assert_eq!(graph.number_edges(), 0);
    assert!(!graph.is_frozen());
}

#[test]
fn test_teloid_graph_with_capacity() {
    let graph = TeloidGraph::with_capacity(10);
    assert!(graph.is_empty());
    assert_eq!(graph.number_nodes(), 0);
    assert_eq!(graph.number_edges(), 0);
    assert!(!graph.is_frozen());
}

#[test]
fn test_add_teloid() {
    let mut graph = TeloidGraph::new();
    let id1: TeloidID = 100;
    let id2: TeloidID = 200;

    let idx1 = graph.add_teloid(id1).unwrap();
    let idx2 = graph.add_teloid(id2).unwrap();

    assert_eq!(idx1, 0);
    assert_eq!(idx2, 1);
    assert_eq!(graph.number_nodes(), 2);
    assert!(graph.contains_teloid(0));
    assert!(graph.contains_teloid(1));
    assert_eq!(graph.get_teloid_id(0), Some(id1));
    assert_eq!(graph.get_teloid_id(1), Some(id2));
}

#[test]
fn test_add_teloid_to_frozen_graph() {
    let mut graph = TeloidGraph::new();
    graph.add_teloid(1).unwrap();
    graph.freeze();
    let result = graph.add_teloid(2);
    assert!(matches!(result, Err(DeonticError::GraphIsFrozen)));
}

#[test]
fn test_get_teloid_id_non_existent() {
    let graph = TeloidGraph::new();
    assert!(graph.get_teloid_id(99).is_none());
}

#[test]
fn test_contains_teloid() {
    let mut graph = TeloidGraph::new();
    graph.add_teloid(100).unwrap();
    assert!(graph.contains_teloid(0));
    assert!(!graph.contains_teloid(1));
}

#[test]
fn test_add_inheritance_edge() {
    let mut graph = TeloidGraph::new();
    let idx1 = graph.add_teloid(1).unwrap();
    let idx2 = graph.add_teloid(2).unwrap();

    graph.add_inheritance_edge(idx1, idx2).unwrap();
    assert_eq!(graph.number_edges(), 1);
    assert!(graph.contains_edge(idx1, idx2));
}

#[test]
fn test_add_defeasance_edge() {
    let mut graph = TeloidGraph::new();
    let idx1 = graph.add_teloid(1).unwrap();
    let idx2 = graph.add_teloid(2).unwrap();

    graph.add_defeasance_edge(idx1, idx2).unwrap();
    assert_eq!(graph.number_edges(), 1);
    assert!(graph.contains_edge(idx1, idx2));

    let tel_id = graph.get_teloid_id(idx1).unwrap();
    assert_eq!(tel_id, 1);
}

#[test]
fn test_add_edge_non_existent_nodes() {
    let mut graph = TeloidGraph::new();
    let result_inherit = graph.add_inheritance_edge(0, 1);
    let result_defeat = graph.add_defeasance_edge(0, 1);

    dbg!(&result_inherit);
    dbg!(&result_defeat);

    assert!(matches!(
        result_inherit,
        Err(DeonticError::FailedToAddEdge(0, 1))
    ));
    assert!(matches!(
        result_defeat,
        Err(DeonticError::FailedToAddEdge(0, 1))
    ));
}

#[test]
fn test_add_edge_to_frozen_graph() {
    let mut graph = TeloidGraph::new();
    let idx1 = graph.add_teloid(1).unwrap();
    let idx2 = graph.add_teloid(2).unwrap();
    graph.freeze();

    let result_inherit = graph.add_inheritance_edge(idx1, idx2);
    let result_defeat = graph.add_defeasance_edge(idx1, idx2);

    assert!(matches!(result_inherit, Err(DeonticError::GraphIsFrozen)));
    assert!(matches!(result_defeat, Err(DeonticError::GraphIsFrozen)));
}

#[test]
fn test_graph_freeze_unfreeze() {
    let mut graph = TeloidGraph::new();
    graph.add_teloid(1).unwrap();
    graph.add_teloid(2).unwrap();
    graph.add_inheritance_edge(0, 1).unwrap();

    graph.freeze();
    assert!(graph.is_frozen());

    graph.unfreeze();
    assert!(!graph.is_frozen());

    // Should be able to add after unfreeze
    graph.add_teloid(3).unwrap();
    assert_eq!(graph.number_nodes(), 3);
}

#[test]
fn test_graph_clear() {
    let mut graph = TeloidGraph::new();
    graph.add_teloid(1).unwrap();
    graph.add_teloid(2).unwrap();
    graph.add_inheritance_edge(0, 1).unwrap();

    graph.clear().unwrap();
    assert!(graph.is_empty());
    assert_eq!(graph.number_nodes(), 0);
    assert_eq!(graph.number_edges(), 0);
}
