/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    BaseTopology, GraphTopology, MixedGraph, MixedGraphTopology, TopologyError, TopologyErrorEnum,
};

fn graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

#[test]
fn base_topology() {
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    assert_eq!(g.dimension(), 1);
    assert_eq!(g.len(), 3);
    assert!(!g.is_empty());
    assert_eq!(g.num_elements_at_grade(0), Some(3));
    assert_eq!(g.num_elements_at_grade(1), Some(1));
    assert_eq!(g.num_elements_at_grade(2), None);
}

#[test]
fn graph_topology_neighbors_span_all_kinds() {
    let mut g = graph(4);
    g.add_arc(0, 1).unwrap();
    g.add_undirected(0, 2).unwrap();
    g.add_bidirected(0, 3).unwrap();
    assert_eq!(g.num_nodes(), 4);
    assert_eq!(GraphTopology::num_edges(&g), 3);
    assert!(g.has_node(3));
    assert!(!g.has_node(4));
    assert_eq!(g.get_neighbors(0).unwrap(), vec![1, 2, 3]);
}

#[test]
fn graph_topology_neighbors_out_of_bounds() {
    let g = graph(2);
    let err = g
        .get_neighbors(9)
        .expect_err("out-of-range node must error");
    assert!(matches!(
        err,
        TopologyError(TopologyErrorEnum::IndexOutOfBounds(_))
    ));
}

#[test]
fn mixed_graph_topology_projection_queries() {
    let mut g = graph(4);
    g.add_arc(1, 0).unwrap(); // 1 → 0
    g.add_arc(2, 0).unwrap(); // 2 → 0
    g.add_undirected(0, 3).unwrap(); // 0 — 3
    assert_eq!(g.num_arcs(), 2);
    assert_eq!(g.num_undirected_edges(), 1);
    assert_eq!(g.get_parents(0).unwrap(), vec![1, 2]);
    assert_eq!(g.get_undirected_neighbors(0).unwrap(), vec![3]);
}

#[test]
fn mixed_graph_topology_out_of_bounds() {
    let g = graph(2);
    assert!(g.get_parents(9).is_err());
    assert!(g.get_undirected_neighbors(9).is_err());
}
