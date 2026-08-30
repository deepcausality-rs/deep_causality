/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Graph, GraphTopology};

// The inherent `Graph::num_edges` getter shadows the `GraphTopology::num_edges`
// trait method, so a plain `graph.num_edges()` call never exercises the trait
// body. These tests call the trait methods through fully-qualified syntax to
// reach the trait implementation.
#[test]
fn test_graph_topology_trait_qualified() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
    let mut graph = Graph::new(4, data, 0).unwrap();
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(1, 2).unwrap();
    graph.add_edge(2, 3).unwrap();

    assert_eq!(<Graph<f64> as GraphTopology>::num_nodes(&graph), 4);
    assert_eq!(<Graph<f64> as GraphTopology>::num_edges(&graph), 3);
    assert!(<Graph<f64> as GraphTopology>::has_node(&graph, 0));
    assert!(!<Graph<f64> as GraphTopology>::has_node(&graph, 4));

    let neighbors = <Graph<f64> as GraphTopology>::get_neighbors(&graph, 2).unwrap();
    assert!(neighbors.contains(&1));
    assert!(neighbors.contains(&3));
}
