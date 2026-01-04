/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Graph;

#[test]
fn test_graph_getters() {
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3]).unwrap();
    let mut graph = Graph::new(3, data, 2).unwrap();
    graph.add_edge(0, 1).unwrap();

    assert_eq!(graph.num_vertices(), 3);
    assert_eq!(graph.num_edges(), 1);

    // Adjacency check
    let adj = graph.adjacencies();
    assert_eq!(adj.len(), 3);
    assert!(adj.get(&0).unwrap().contains(&1));
    assert!(adj.get(&1).unwrap().contains(&0));
    assert!(adj.get(&2).unwrap().is_empty());

    // Data check
    assert_eq!(graph.data().len(), 3);

    // Cursor check
    assert_eq!(graph.cursor(), 2);
}
