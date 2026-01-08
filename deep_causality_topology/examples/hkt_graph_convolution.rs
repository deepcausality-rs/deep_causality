/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{CoMonad, Functor};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Graph, GraphWitness};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HKT Graph Convolution (GNN) Example ===\n");

    // ------------------------------------------------------------------------
    // ENGINEERING VALUE:
    // Graph Neural Networks (GNNs) and Cellular Automata rely on "local" operations
    // applied globally. A node's new state depends on its current state and the
    // state of its neighbors.
    //
    // In Functional Programming, this pattern is captured by the **Comonad**.
    // - `extract`: Get the value at the current focus (node).
    // - `extend`: Apply a context-aware function to *every* position in the structure
    //             to generate a new structure.
    //
    // This example demonstrates using `BoundedComonad::extend` to implement a
    // single layer of a Graph Convolution Network (GCN) or a diffusion step
    // without writing explicit loops over nodes. The HKT abstraction handles
    // the iteration and context management (cursor movement).
    // ------------------------------------------------------------------------

    // 1. Setup the Graph (Social Network / Sensor Grid)
    // 4 Nodes: 0-1, 1-2, 2-3, 3-0 (Ring) + 1-3 (Cross connection)
    // 0 -- 1
    // |    | \
    // 3 -- 2
    // 1. Setup the Graph (Social Network / Sensor Grid)
    // 4 Nodes: 0-1, 1-2, 2-3, 3-0 (Ring) + 1-3 (Cross connection)
    // 0 -- 1
    // |    | \
    // 3 -- 2
    let num_nodes = 4;

    // Initial Signal (e.g., Heat, Voltage, or Feature Vector)
    // Node 0 is "Hot" (10.0), others are "Cold" (0.0)
    let initial_data = CausalTensor::new(vec![10.0, 0.0, 0.0, 0.0], vec![4])?;
    let mut graph = Graph::new(num_nodes, initial_data, 0)?;

    // Add edges manually
    graph.add_edge(0, 1)?;
    graph.add_edge(1, 2)?;
    graph.add_edge(2, 3)?;
    graph.add_edge(3, 0)?;
    graph.add_edge(1, 3)?;

    println!("--- Initial State ---");
    print_graph_state(&graph);

    // 2. Define the Convolution Kernel (The "Local Rule")
    // This function takes a "View" of the graph focused at a specific node.
    // It returns the new value for that node.
    // Rule: New Value = (Self + Sum(Neighbors)) / (1 + Num_Neighbors)
    // This is a simple mean-pooling diffusion.
    let diffusion_kernel = |g: &Graph<f64>| {
        // 1. Get value of focused node (Self)
        let current_val = GraphWitness::extract(g);

        // 2. Get neighbors of focused node
        // Note: `g.cursor()` tells us which node is currently in focus.
        let cursor = g.cursor();
        // Use a static empty vec for fallback to avoid temporary lifetime issue
        static EMPTY_VEC: Vec<usize> = Vec::new();
        let neighbors = g.neighbors(cursor).unwrap_or(&EMPTY_VEC);

        // 3. Sum neighbor values
        let mut sum_neighbors = 0.0;
        let data_slice = g.data().as_slice();
        for &n_idx in neighbors {
            if let Some(&val) = data_slice.get(n_idx) {
                sum_neighbors += val;
            }
        }

        // 4. Compute Average
        let count = 1.0 + neighbors.len() as f64;
        (current_val + sum_neighbors) / count
    };

    // 3. Apply Convolution (Extend)
    // This applies the kernel to EVERY node automatically.
    println!("\n--- Step 1: Diffusion (Extend) ---");
    let step1_graph = GraphWitness::extend(&graph, diffusion_kernel);
    print_graph_state(&step1_graph);

    // 4. Apply Activation / Normalization (Functor)
    // Apply a ReLU-like activation or just scale it.
    // Let's say we want to amplify weak signals: if x < 1.0 { 0.0 } else { x * 1.1 }
    println!("\n--- Step 2: Activation (Functor) ---");
    let step2_graph = GraphWitness::fmap(step1_graph, |x| if x < 0.1 { 0.0 } else { x });
    print_graph_state(&step2_graph);

    // 5. Another Diffusion Step
    println!("\n--- Step 3: Diffusion (Extend) ---");
    let step3_graph = GraphWitness::extend(&step2_graph, diffusion_kernel);
    print_graph_state(&step3_graph);

    Ok(())
}

fn print_graph_state(g: &Graph<f64>) {
    let data = g.data().as_slice();
    println!("Node Values: {:?}", data);
}
