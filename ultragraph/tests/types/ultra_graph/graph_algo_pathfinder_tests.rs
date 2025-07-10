/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use ultragraph::{
    GraphError, GraphMut, PathfindingGraphAlgorithms, UltraGraph, UltraGraphWeighted,
};

use ultragraph::utils::utils_tests;
#[test]
fn test_is_reachable_on_dynamic_graph() {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    assert!(matches!(
        g.is_reachable(0, 1),
        Err(GraphError::GraphNotFrozen)
    ));
}

#[test]
fn test_is_reachable_on_static_graph() {
    let g = utils_tests::get_acyclic_graph();
    assert!(g.is_reachable(0, 3).unwrap());
    assert!(!g.is_reachable(3, 0).unwrap());
}

#[test]
fn test_is_reachable_invalid_node() {
    let g = utils_tests::get_acyclic_graph();
    let res = g.is_reachable(0, 99);
    assert!(res.is_ok());
    let reachable = res.unwrap();
    assert!(!reachable);

    let res = g.is_reachable(99, 0);
    assert!(res.is_ok());
    let reachable = res.unwrap();
    assert!(!reachable);
}

#[test]
fn test_shortest_path_len_on_static_graph() {
    let g = utils_tests::get_acyclic_graph();
    assert_eq!(g.shortest_path_len(0, 3).unwrap(), Some(4));
    assert_eq!(g.shortest_path_len(3, 0).unwrap(), None);
}

#[test]
fn test_shortest_path_len_is_one() {
    let g = utils_tests::get_acyclic_graph();
    let res = g.shortest_path_len(0, 0);
    assert!(res.is_ok());
    let res = res.unwrap();
    assert_eq!(res, Some(1));
}

#[test]
fn test_shortest_path_on_dynamic_graph() {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    assert!(matches!(
        g.shortest_path(0, 1),
        Err(GraphError::GraphNotFrozen)
    ));
}

#[test]
fn test_shortest_path_is_one() {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap();
    g.freeze();

    let res = g.shortest_path(0, 0);
    assert!(res.is_ok());
    let res = res.unwrap();
    assert_eq!(res, Some(vec![0]));
}

#[test]
fn test_shortest_path_on_static_graph() {
    let g = utils_tests::get_acyclic_graph();
    assert_eq!(g.shortest_path(0, 3).unwrap(), Some(vec![0, 1, 2, 3]));
    assert_eq!(g.shortest_path(3, 0).unwrap(), None);
}

#[test]
fn test_shortest_weighted_path_on_static_graph() {
    let mut g = UltraGraphWeighted::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    g.add_node(3).unwrap();
    g.add_node(4).unwrap();
    g.add_edge(0, 1, 1).unwrap();
    g.add_edge(0, 2, 4).unwrap();
    g.add_edge(1, 2, 2).unwrap();
    g.add_edge(1, 3, 5).unwrap();
    g.add_edge(2, 3, 1).unwrap();
    g.add_edge(3, 4, 1).unwrap();

    let result = g.shortest_weighted_path(0, 4);
    // triggers GraphNotFrozen error.
    assert!(result.is_err());

    // Freeze the graph to enable all algos.
    g.freeze();

    // Path 0 -> 1 -> 2 -> 3 -> 4, weight 1 + 2 + 1 + 1 = 5
    let result = g.shortest_weighted_path(0, 4).unwrap().unwrap();
    assert_eq!(result.0, vec![0, 1, 2, 3, 4]);
    assert_eq!(result.1, 5);

    // Path 0 -> 1 -> 2 -> 3, weight 1 + 2 + 1 = 4
    let result = g.shortest_weighted_path(0, 3).unwrap().unwrap();
    assert_eq!(result.0, vec![0, 1, 2, 3]);
    assert_eq!(result.1, 4);

    // Path 1 -> 2 -> 3, weight 2 + 1 = 3
    let result = g.shortest_weighted_path(1, 3).unwrap().unwrap();
    assert_eq!(result.0, vec![1, 2, 3]);
    assert_eq!(result.1, 3);

    // No path from 4 to 0
    assert!(g.shortest_weighted_path(4, 0).unwrap().is_none());

    // Invalid nodes
    assert!(g.shortest_weighted_path(0, 99).unwrap().is_none());
    assert!(g.shortest_weighted_path(99, 0).unwrap().is_none());

    // Start and stop are the same
    let result = g.shortest_weighted_path(0, 0).unwrap().unwrap();
    assert_eq!(result.0, vec![0]);
    assert_eq!(result.1, 0);
}
