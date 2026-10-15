/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use ultragraph::utils::utils_tests;
use ultragraph::{
    GraphError, GraphMut, GraphView, TopologicalGraphAlgorithms, UltraGraph, UltraGraphContainer,
};

#[test]
fn test_find_cycle_on_dynamic_graph() {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap();
    assert!(matches!(g.find_cycle(), Err(GraphError::GraphNotFrozen)));
}

#[test]
fn test_find_cycle_on_empty_graph() {
    let mut g: UltraGraphContainer<u64, ()> = UltraGraph::new();
    g.freeze();
    assert!(matches!(g.find_cycle(), Ok(None)));
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

#[test]
fn test_topological_sort_on_empty_graph() {
    let mut g: UltraGraphContainer<u64, ()> = UltraGraph::new();
    g.freeze();
    let res = g.topological_sort();
    dbg!(&res);
    assert!(&res.is_ok());
    assert!(&res.unwrap().is_some());
}

/// The cycle is returned as a closed walk that starts and ends on the same node: for the
/// triangle 0 -> 1 -> 2 -> 0 the depth-first search from 0 reports `[0, 1, 2, 0]`.
#[test]
fn test_find_cycle_returns_the_closed_walk() {
    let mut g = UltraGraph::new();
    for payload in 0..4 {
        g.add_node(payload).unwrap();
    }
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(2, 0, ()).unwrap();
    g.add_edge(3, 0, ()).unwrap();
    g.freeze();

    assert_eq!(g.find_cycle().unwrap(), Some(vec![0, 1, 2, 0]));
}

/// A self-loop on node 1 is the walk `[1, 1]`.
#[test]
fn test_find_cycle_reports_a_self_loop() {
    let mut g = UltraGraph::new();
    for payload in 0..2 {
        g.add_node(payload).unwrap();
    }
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(1, 1, ()).unwrap();
    g.freeze();

    assert_eq!(g.find_cycle().unwrap(), Some(vec![1, 1]));
}

/// A diamond reaches node 3 twice without a cycle: a node met again after it was finished is not
/// a back edge.
#[test]
fn test_find_cycle_on_a_diamond_is_none() {
    let mut g = UltraGraph::new();
    for payload in 0..4 {
        g.add_node(payload).unwrap();
    }
    for (a, b) in [(0, 1), (0, 2), (1, 3), (2, 3)] {
        g.add_edge(a, b, ()).unwrap();
    }
    g.freeze();

    assert_eq!(g.find_cycle().unwrap(), None);
    assert!(!g.has_cycle().unwrap());
}
