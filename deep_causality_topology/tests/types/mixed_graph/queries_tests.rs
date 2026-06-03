/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{EdgeKind, Mark, MixedGraph};

fn graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

#[test]
fn parents_and_undirected_neighbors_are_separate() {
    // 0 → 2, 1 — 2
    let mut g = graph(3);
    g.add_arc(0, 2).unwrap();
    g.add_undirected(1, 2).unwrap();
    assert_eq!(g.parents(2), vec![0]);
    assert_eq!(g.undirected_neighbors(2), vec![1]);
    // 0 is a parent, not an undirected neighbor; 1 vice versa
    assert!(!g.parents(2).contains(&1));
    assert!(!g.undirected_neighbors(2).contains(&0));
}

#[test]
fn children_reports_outgoing_arcs() {
    // 1 → 0, 1 → 2
    let mut g = graph(3);
    g.add_arc(1, 0).unwrap();
    g.add_arc(1, 2).unwrap();
    assert_eq!(g.children(1), vec![0, 2]);
    assert_eq!(g.parents(0), vec![1]);
    assert_eq!(g.parents(2), vec![1]);
    assert!(g.children(0).is_empty());
}

#[test]
fn parents_children_consistent_across_node_order() {
    // arc added as (2, 1): 2 → 1
    let mut g = graph(3);
    g.add_arc(2, 1).unwrap();
    assert_eq!(g.parents(1), vec![2]);
    assert_eq!(g.children(2), vec![1]);
}

#[test]
fn adjacency_spans_all_edge_kinds() {
    let mut g = graph(4);
    g.add_arc(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g.add_bidirected(2, 3).unwrap();
    assert!(g.is_adjacent(0, 1));
    assert!(g.is_adjacent(1, 0)); // order-agnostic
    assert!(g.is_adjacent(1, 2));
    assert!(g.is_adjacent(2, 3));
    assert!(!g.is_adjacent(0, 3));
}

#[test]
fn arcs_enumerated_as_parent_child() {
    let mut g = graph(3);
    g.add_arc(2, 0).unwrap(); // stored canonical (0,2) with Arrow at 0
    g.add_arc(1, 2).unwrap();
    g.add_undirected(0, 1).unwrap(); // not an arc
    let mut arcs = g.arcs();
    arcs.sort_unstable();
    assert_eq!(arcs, vec![(1, 2), (2, 0)]);
}

#[test]
fn enumeration_by_kind() {
    let mut g = graph(4);
    g.add_arc(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g.add_undirected(2, 3).unwrap();
    g.add_bidirected(0, 3).unwrap();
    assert_eq!(g.edges_of_kind(EdgeKind::Directed), vec![(0, 1)]);
    assert_eq!(g.undirected_edges(), vec![(1, 2), (2, 3)]);
    assert_eq!(g.edges_of_kind(EdgeKind::Bidirected), vec![(0, 3)]);
    assert_eq!(
        g.edges_of_kind(EdgeKind::PartiallyDirected),
        Vec::<(usize, usize)>::new()
    );
}

#[test]
fn partially_directed_edge_is_not_an_arc() {
    // ∘→ is not a (compelled) directed arc in the projection
    let mut g = graph(2);
    g.add_edge(0, 1, Mark::Circle, Mark::Arrow).unwrap();
    assert!(g.arcs().is_empty());
    assert!(g.parents(1).is_empty());
    assert!(g.is_adjacent(0, 1));
}
