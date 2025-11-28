/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Graph, GraphTopology};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Graph Example ===");

    // ------------------------------------------------------------------------
    // ENGINEERING VALUE:
    // Graphs are the fundamental data structure for modeling relationships,
    // dependencies, and causal links in complex systems.
    //
    // This example demonstrates the core API for:
    // 1. Constructing efficient graph structures using sparse matrices.
    // 2. Managing node metadata with causal tensors.
    // 3. performing basic traversal and structural queries.
    //
    // Using sparse matrices (CSR) for adjacency ensures memory efficiency
    // even for large-scale causal models with millions of nodes.
    // ------------------------------------------------------------------------

    // 1. Define Graph Parameters
    let num_vertices = 3;

    // 2. Create Data Tensor (Node values)
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3])?;

    // 3. Initialize Graph (Empty edges initially)
    let mut graph = Graph::new(num_vertices, data, 0)?;
    println!("Graph Created: {}", graph);

    // 4. Add Edges (Cycle: 0->1, 1->2, 2->0)
    // Note: add_edge adds undirected edge (u,v) and (v,u)
    graph.add_edge(0, 1)?;
    graph.add_edge(1, 2)?;
    graph.add_edge(2, 0)?;

    // 5. Inspect Topology
    println!("Number of Nodes: {}", graph.num_nodes());
    println!("Number of Edges: {}", graph.num_edges());

    // 6. Check Edges
    println!("Has edge 0->1? {:?}", graph.has_edge(0, 1)?);
    println!("Has edge 0->2? {:?}", graph.has_edge(0, 2)?);

    // 7. Get Neighbors
    let neighbors_of_1 = graph.get_neighbors(1)?;
    println!("Neighbors of Node 1: {:?}", neighbors_of_1);

    // 8. Add a new edge (0->2) - Already exists due to cycle 2->0 and undirected nature?
    // Let's check. 2->0 added edge (2,0) and (0,2).
    // So 0->2 should already exist.
    println!("Has edge 0->2 before adding? {:?}", graph.has_edge(0, 2)?);

    // Attempt to add again
    let added = graph.add_edge(0, 2)?;
    println!("Added edge 0->2 again? {}", added); // Should be false

    Ok(())
}
