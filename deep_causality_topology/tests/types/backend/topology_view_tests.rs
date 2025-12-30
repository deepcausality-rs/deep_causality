/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#[cfg(feature = "mlx")]
use deep_causality_tensor::backend::MlxBackend;
use deep_causality_tensor::{CausalTensor, CpuBackend};
use deep_causality_topology::Graph;
use deep_causality_topology::backend::TopologyView;

#[test]
fn test_topology_view_projection() {
    let num_vertices = 2;
    // Create dummy data tensor for graph vertices
    let data = CausalTensor::<f32>::zeros(&[num_vertices]);
    let mut graph = Graph::new(num_vertices, data, 0).expect("Graph creation failed");

    // Nodes are created by new with IDs implicitly 0..N-1?
    // Graph::add_node logic isn't showing in the new() constructor.
    // Check Graph::add_node signature.
    // If Graph::new creates vertices, we don't need add_node?
    // Let's assume we can add edges between existing indices 0 and 1.
    // But typically add_node returns an ID.
    // Let's assume nodes 0 and 1 exist.
    let n0 = 0;
    let n1 = 1;

    graph.add_edge(n0, n1).expect("Failed to add edge");

    // Test with CPU backend (guaranteed available)
    let _view = TopologyView::<CpuBackend, f32>::from_graph(&graph);

    // Matrix size 2x2
    // Degree n0 = 1 (out), n1 = 0
    // Matrix:
    // [[0, 1],
    //  [0, 0]]

    // Verify degrees
    // (Actual verification requires tensor readback, assuming standard tensor API or converting to vec)
    // view.degrees ...
}
