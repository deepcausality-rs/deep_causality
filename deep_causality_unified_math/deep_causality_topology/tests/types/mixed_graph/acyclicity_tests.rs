/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

fn graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

/// Checks that `order` places every arc's parent before its child.
fn is_valid_topo(g: &MixedGraph<()>, order: &[usize]) -> bool {
    let mut pos = vec![0usize; g.num_vertices()];
    for (i, &v) in order.iter().enumerate() {
        pos[v] = i;
    }
    g.arcs().iter().all(|&(p, c)| pos[p] < pos[c])
}

#[test]
fn acyclic_projection_yields_valid_order() {
    // 0 → 1 → 2, 0 → 2
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_arc(0, 2).unwrap();
    let order = g
        .topological_sort()
        .expect("acyclic graph has a topological order");
    assert_eq!(order.len(), 3);
    assert!(is_valid_topo(&g, &order));
    assert!(!g.has_cycle());
    assert_eq!(g.find_cycle(), None);
}

#[test]
fn deterministic_smallest_first_order() {
    // Independent nodes 0,1,2 plus arc 2 → 0: ready set is {1,2}, smallest first.
    let mut g = graph(3);
    g.add_arc(2, 0).unwrap();
    assert_eq!(g.topological_sort(), Some(vec![1, 2, 0]));
}

#[test]
fn cyclic_projection_is_detected() {
    // 0 → 1 → 2 → 0
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_arc(2, 0).unwrap();
    assert_eq!(g.topological_sort(), None);
    assert!(g.has_cycle());
    let cycle = g.find_cycle().expect("a cycle exists");
    assert_eq!(cycle.len(), 3);
    // The reported nodes are exactly the cycle members.
    let mut sorted = cycle.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, vec![0, 1, 2]);
}

#[test]
fn undirected_edges_do_not_affect_acyclicity() {
    // Acyclic arcs 0 → 1 → 2, plus undirected 0 — 2 and 1 — 2.
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_undirected(0, 2).unwrap();
    let order = g
        .topological_sort()
        .expect("undirected edges are ignored by acyclicity");
    assert!(is_valid_topo(&g, &order));
    assert!(!g.has_cycle());
}

#[test]
fn isolated_and_undirected_only_nodes_appear_in_order() {
    // node 0 isolated; 1 — 2 undirected only; no arcs at all.
    let mut g = graph(3);
    g.add_undirected(1, 2).unwrap();
    let order = g.topological_sort().expect("no arcs => acyclic");
    assert_eq!(order.len(), 3);
    let mut sorted = order.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, vec![0, 1, 2]);
}

#[test]
fn bidirected_edges_are_not_arcs_and_stay_acyclic() {
    // 0 ↔ 1 is not a directed arc; the projection is empty, hence acyclic.
    let mut g = graph(2);
    g.add_bidirected(0, 1).unwrap();
    assert!(!g.has_cycle());
    assert_eq!(g.topological_sort(), Some(vec![0, 1]));
}
