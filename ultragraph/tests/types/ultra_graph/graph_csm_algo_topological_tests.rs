/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use ultragraph::utils::utils_tests;
use ultragraph::{GraphError, GraphMut, GraphView, TopologicalGraphAlgorithms, UltraGraph};

#[test]
fn test_find_cycle_on_dynamic_graph() {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap();
    assert!(matches!(g.find_cycle(), Err(GraphError::GraphNotFrozen)));
}

#[test]
fn test_find_cycle_on_cyclic_graph() {
    let g = utils_tests::get_cyclic_graph();
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
    let g = utils_tests::get_acyclic_graph();
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
    let g = utils_tests::get_cyclic_graph();
    assert!(g.has_cycle().unwrap());
}

#[test]
fn test_has_cycle_on_acyclic_graph() {
    let g = utils_tests::get_acyclic_graph();
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
    let g = utils_tests::get_cyclic_graph();
    assert!(g.topological_sort().unwrap().is_none());
}

#[test]
fn test_topological_sort_on_acyclic_graph() {
    let g = utils_tests::get_acyclic_graph();
    let sorted = g.topological_sort().unwrap().unwrap();
    assert_eq!(sorted, vec![0, 1, 2, 3]);
}
