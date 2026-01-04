/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, Graph};

#[test]
fn test_graph_base_topology() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let mut graph = Graph::new(3, data, 0).unwrap();
    graph.add_edge(0, 1).unwrap();

    // Graph is 1-dimensional
    assert_eq!(graph.dimension(), 1);

    // len returns number of vertices
    assert_eq!(graph.len(), 3);

    // Check elements at grade
    assert_eq!(graph.num_elements_at_grade(0), Some(3)); // 3 vertices
    assert_eq!(graph.num_elements_at_grade(1), Some(1)); // 1 edge
    assert_eq!(graph.num_elements_at_grade(2), None);

    assert!(!graph.is_empty());
}
