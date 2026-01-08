/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Graph;

#[test]
fn test_graph_display() {
    let data = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let mut graph = Graph::new(2, data, 0).unwrap();
    graph.add_edge(0, 1).unwrap();

    let output = format!("{}", graph);
    // Expected: Graph { vertices: 2, edges: 1 }
    assert!(output.contains("Graph"));
    assert!(output.contains("vertices: 2"));
    assert!(output.contains("edges: 1"));
}
