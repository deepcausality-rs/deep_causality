/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Graph x Tensor: a convolution layer via comonadic extension
//!
//! A graph neural network layer and a cellular automaton share one shape: the new value at a
//! node depends on that node and its neighbours. `CoMonad` is that shape. `extract` reads the
//! focused node, and `extend` applies a neighbourhood-aware kernel at every position at once,
//! so the layer is written without a loop over nodes.
//!
//! Here the kernel is mean pooling, `(self + sum(neighbours)) / (1 + degree)`, which is a
//! diffusion step. Topology supplies the walk and the adjacency; the tensor holds the node
//! features; the kernel decides what a neighbourhood means. `fmap` then applies the activation,
//! because an activation is per-node and needs no neighbourhood at all.
//!
//! ## APIs Demonstrated
//! - `Graph::new`, `Graph::add_edge`, `Graph::neighbors`
//! - `GraphWitness::extract` and `GraphWitness::extend` (CoMonad)
//! - `GraphWitness::fmap` (Functor)

use deep_causality_haft::{CoMonad, Functor};
use deep_causality_num::{Lift, const_scalar_from_float, const_scalar_from_int, lower};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Graph, GraphWitness};

/// A ring `0-1-2-3-0` with one cross connection `1-3`.
const N_NODES: usize = 4;
const EDGES: [(usize, usize); 5] = [(0, 1), (1, 2), (2, 3), (3, 0), (1, 3)];

/// The signal at node 0 before the first step; every other node starts at zero.
const SOURCE_VALUE: FloatType = const_scalar_from_int!(FloatType, 10);

/// The activation floor: a node below this is driven to zero.
const ACTIVATION_THRESHOLD: FloatType = const_scalar_from_float!(FloatType, 0.1);

/// `f64` is the right precision here: four nodes and two diffusion steps, so the pooled
/// averages stay far from any rounding limit. The kernel itself flows through any `RealField`
/// implementor.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let graph = build_graph()?;
    print_stage("--- Initial State ---", &graph);

    // One `extend` applies the kernel at every node. The witness walks the positions; the
    // kernel says what one position means.
    let diffused = GraphWitness::extend(&graph, diffusion_kernel);
    print_stage("\n--- Step 1: Diffusion (Extend) ---", &diffused);

    // The activation is per-node, so it needs `fmap` rather than `extend`.
    let threshold = ACTIVATION_THRESHOLD;
    let activated = GraphWitness::fmap(diffused, move |x| if x < threshold { ZERO } else { x });
    print_stage("\n--- Step 2: Activation (Functor) ---", &activated);

    let diffused_again = GraphWitness::extend(&activated, diffusion_kernel);
    print_stage("\n--- Step 3: Diffusion (Extend) ---", &diffused_again);

    Ok(())
}

/// The ring plus cross connection, carrying the initial signal on its nodes.
fn build_graph() -> Result<Graph<FloatType>, Box<dyn std::error::Error>> {
    let zero = ZERO;
    let mut features = vec![zero; N_NODES];
    features[0] = SOURCE_VALUE;

    let initial = CausalTensor::new(features, vec![N_NODES])?;
    let mut graph = Graph::new(N_NODES, initial, 0)?;
    for (u, v) in EDGES {
        graph.add_edge(u, v)?;
    }
    Ok(graph)
}

/// Mean pooling at the focused node: `(self + sum(neighbours)) / (1 + degree)`.
///
/// `extract` reads the focused value and `cursor` names the position, which is what lets the
/// kernel reach the adjacency for that node alone.
fn diffusion_kernel(g: &Graph<FloatType>) -> FloatType {
    let current = GraphWitness::extract(g);
    let neighbors = g.neighbors(g.cursor()).map(Vec::as_slice).unwrap_or(&[]);

    let features = g.data().as_slice();
    let pooled = neighbors
        .iter()
        .filter_map(|&n| features.get(n))
        .fold(current, |acc, &v| acc + v);

    pooled / (ONE + neighbors.len().lift::<FloatType>())
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Graph x Tensor: a Convolution Layer via Comonadic Extension ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

fn print_stage(title: &str, g: &Graph<FloatType>) {
    println!("{title}");
    println!("Node Values: {:?}", shown(g));
}

/// The display boundary: `f64` appears here and nowhere else.
fn shown(g: &Graph<FloatType>) -> Vec<f64> {
    g.data().as_slice().iter().map(|&v| lower(v)).collect()
}
