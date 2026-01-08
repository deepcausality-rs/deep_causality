/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Graph, GraphTopology};

#[test]
fn test_graph_topology_trait() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
    let mut graph = Graph::new(4, data, 0).unwrap();
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(1, 2).unwrap();

    assert_eq!(graph.num_nodes(), 4);
    assert_eq!(graph.num_edges(), 2);

    assert!(graph.has_node(0));
    assert!(graph.has_node(3));
    assert!(!graph.has_node(4)); // 0-indexed, so 4 is out of bounds

    // Neighbors
    let neighbors_1 = graph.get_neighbors(1).unwrap();
    assert_eq!(neighbors_1.len(), 2);
    assert!(neighbors_1.contains(&0));
    assert!(neighbors_1.contains(&2));

    // Error case
    let err = graph.get_neighbors(5);
    assert!(err.is_err());
}
