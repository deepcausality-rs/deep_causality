/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Graph;

#[test]
fn test_graph_clone_shallow() {
    let data = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let mut graph = Graph::new(2, data, 1).unwrap();
    graph.add_edge(0, 1).unwrap();

    // Clone shallow should reset cursor to 0
    let cloned = graph.clone_shallow();

    assert_eq!(cloned.num_vertices(), 2);
    assert_eq!(cloned.num_edges(), 1);
    assert_eq!(cloned.cursor(), 0); // Reset
    assert_eq!(graph.cursor(), 1); // Original unchanged

    // Verify structure preservation
    assert!(cloned.has_edge(0, 1).unwrap());
}
