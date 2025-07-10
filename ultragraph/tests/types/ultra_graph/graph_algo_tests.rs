/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::*;

// Helper to create a graph for testing.
// 0 -> 1 -> 2
// ^----|    |-> 3
// |         |
// +---------+
fn get_cyclic_graph() -> UltraGraphContainer<i32, ()> {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    g.add_node(3).unwrap();
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(2, 0, ()).unwrap(); // Cycle
    g.add_edge(2, 3, ()).unwrap();
    g.freeze();
    g
}

// Helper to create a DAG.
// 0 -> 1 -> 2 -> 3
fn get_acyclic_graph() -> UltraGraphContainer<i32, ()> {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    g.add_node(3).unwrap();
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(2, 3, ()).unwrap();
    g.freeze();
    g
}

#[test]
fn test_find_cycle_on_dynamic_graph() {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap();
    assert!(matches!(g.find_cycle(), Err(GraphError::GraphNotFrozen)));
}

#[test]
fn test_find_cycle_on_cyclic_graph() {
    let g = get_cyclic_graph();
    let cycle = g.find_cycle().unwrap();
    assert!(cycle.is_some());
    let path = cycle.unwrap();
    assert!(
        path.windows(2).all(|w| g.contains_edge(w[0], w[1]))
            || g.contains_edge(path[path.len() - 1], path[0])
    );
}

#[test]
fn test_find_cycle_on_acyclic_graph() {
    let g = get_acyclic_graph();
    assert!(g.find_cycle().unwrap().is_none());
}

#[test]
fn test_has_cycle_on_dynamic_graph() {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap();
    assert!(matches!(g.has_cycle(), Err(GraphError::GraphNotFrozen)));
}

#[test]
fn test_has_cycle_on_cyclic_graph() {
    let g = get_cyclic_graph();
    assert!(g.has_cycle().unwrap());
}

#[test]
fn test_has_cycle_on_acyclic_graph() {
    let g = get_acyclic_graph();
    assert!(!g.has_cycle().unwrap());
}

#[test]
fn test_topological_sort_on_dynamic_graph() {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap();
    assert!(matches!(
        g.topological_sort(),
        Err(GraphError::GraphNotFrozen)
    ));
}

#[test]
fn test_topological_sort_on_cyclic_graph() {
    let g = get_cyclic_graph();
    assert!(g.topological_sort().unwrap().is_none());
}

#[test]
fn test_topological_sort_on_acyclic_graph() {
    let g = get_acyclic_graph();
    let sorted = g.topological_sort().unwrap().unwrap();
    assert_eq!(sorted, vec![0, 1, 2, 3]);
}

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
    let g = get_acyclic_graph();
    assert!(g.is_reachable(0, 3).unwrap());
    assert!(!g.is_reachable(3, 0).unwrap());
}

#[test]
fn test_is_reachable_invalid_node() {
    let g = get_acyclic_graph();
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
    let g = get_acyclic_graph();
    assert_eq!(g.shortest_path_len(0, 3).unwrap(), Some(4));
    assert_eq!(g.shortest_path_len(3, 0).unwrap(), None);
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
fn test_shortest_path_on_static_graph() {
    let g = get_acyclic_graph();
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
