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

#[test]
fn test_strongly_connected_components() {
    let mut g = UltraGraphWeighted::new();
    // SCC 1: {0, 1, 2}
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(2, 0, ()).unwrap();

    // SCC 2: {3, 4}
    g.add_node(3).unwrap();
    g.add_node(4).unwrap();
    g.add_edge(3, 4, ()).unwrap();
    g.add_edge(4, 3, ()).unwrap();

    // SCC 3: {5, 6}
    g.add_node(5).unwrap();
    g.add_node(6).unwrap();
    g.add_edge(5, 6, ()).unwrap();
    g.add_edge(6, 5, ()).unwrap();

    // SCC 4: {7} (self-loop)
    g.add_node(7).unwrap();
    g.add_edge(7, 7, ()).unwrap();

    // SCC 5: {8} (isolated node)
    g.add_node(8).unwrap();

    // Edges connecting SCCs (should not merge them)
    g.add_edge(2, 3, ()).unwrap();
    g.add_edge(4, 5, ()).unwrap();
    g.add_edge(6, 7, ()).unwrap();

    g.freeze();

    let mut sccs = g.strongly_connected_components().unwrap();

    // Sort each SCC internally and then sort the list of SCCs for consistent comparison
    for scc in sccs.iter_mut() {
        scc.sort_unstable();
    }
    sccs.sort_unstable_by_key(|scc| scc[0]); // Sort by the first element of each SCC

    let mut expected_sccs = vec![vec![0, 1, 2], vec![3, 4], vec![5, 6], vec![7], vec![8]];
    for scc in expected_sccs.iter_mut() {
        scc.sort_unstable();
    }
    expected_sccs.sort_unstable_by_key(|scc| scc[0]);

    assert_eq!(sccs, expected_sccs);

    // Test with an empty graph
    let mut g_empty = UltraGraphWeighted::<i32, i32>::new();
    g_empty.freeze();
    let sccs_empty = g_empty.strongly_connected_components().unwrap();
    assert!(sccs_empty.is_empty());

    // Test with a single node graph
    let mut g_single = UltraGraph::new();
    g_single.add_node(0).unwrap();
    g_single.freeze();
    let sccs_single = g_single.strongly_connected_components().unwrap();
    assert_eq!(sccs_single, vec![vec![0]]);

    // Test with a DAG
    let g_dag = get_acyclic_graph();
    let mut sccs_dag = g_dag.strongly_connected_components().unwrap();
    for scc in sccs_dag.iter_mut() {
        scc.sort_unstable();
    }
    sccs_dag.sort_unstable_by_key(|scc| scc[0]);
    let mut expected_sccs_dag = vec![vec![0], vec![1], vec![2], vec![3]];
    for scc in expected_sccs_dag.iter_mut() {
        scc.sort_unstable();
    }
    expected_sccs_dag.sort_unstable_by_key(|scc| scc[0]);
    assert_eq!(sccs_dag, expected_sccs_dag);
}
