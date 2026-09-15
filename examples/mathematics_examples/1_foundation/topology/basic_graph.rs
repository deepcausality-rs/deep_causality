/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `Graph`: the graph API surface
//!
//! Graphs model relationships, dependencies and causal links. Adjacency is stored as a sparse
//! CSR matrix, so memory stays proportional to the edges rather than to the square of the
//! nodes, and node payload rides in a `CausalTensor` alongside it.
//!
//! `add_edge` is undirected: adding `(u, v)` adds `(v, u)` too, and re-adding an edge that is
//! already present returns `false` rather than duplicating it.

use deep_causality_num::lift;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Graph, GraphTopology};

/// The working scalar. Node payload carries it.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A three-node graph whose payload is one value per node.
    let data = CausalTensor::new(
        vec![lift::<FloatType>(10.0), lift(20.0), lift(30.0)],
        vec![3],
    )?;
    let mut graph = Graph::new(3, data, 0)?;
    print_created(&graph);

    // A cycle: 0-1, 1-2, 2-0. Each call adds both directions.
    graph.add_edge(0, 1)?;
    graph.add_edge(1, 2)?;
    graph.add_edge(2, 0)?;

    print_topology(graph.num_nodes(), graph.num_edges());
    print_edges(graph.has_edge(0, 1)?, graph.has_edge(0, 2)?);
    print_neighbors(1, &graph.get_neighbors(1)?);

    // 2-0 already put (0, 2) in place, so adding it again is a no-op that reports `false`.
    let existed = graph.has_edge(0, 2)?;
    let added = graph.add_edge(0, 2)?;
    print_duplicate(existed, added);

    Ok(())
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_created(graph: &Graph<FloatType>) {
    println!("=== Basic Graph Example ===");
    println!("Graph Created: {graph}");
}

fn print_topology(nodes: usize, edges: usize) {
    println!("Number of Nodes: {nodes}");
    println!("Number of Edges: {edges}");
}

fn print_edges(zero_one: bool, zero_two: bool) {
    println!("Has edge 0->1? {zero_one:?}");
    println!("Has edge 0->2? {zero_two:?}");
}

fn print_neighbors(node: usize, neighbors: &[usize]) {
    println!("Neighbors of Node {node}: {neighbors:?}");
}

fn print_duplicate(existed: bool, added: bool) {
    println!("Has edge 0->2 before adding? {existed:?}");
    println!("Added edge 0->2 again? {added}");
}
