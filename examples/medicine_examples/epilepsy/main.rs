/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Virtual Epilepsy Surgery Planning (Digital Twin)
//!
//! Demonstrates using Network Topology and Causal Interventions to plan epilepsy surgery.
//!
//! ## Key Concepts
//! - **Connectome Topology**: Brain regions modeled as a Graph.
//! - **Seizure Dynamics**: Kuramoto oscillators simulating synchronization.
//! - **Virtual Resection**: Causal intervention `do(remove_node)` to find curative surgeries.
use deep_causality_calculus::{EndoArrow, Euler};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Graph;
use std::f64::consts::PI;
use std::ops::{Add, Mul};

// Constants
const SEIZURE_THRESHOLD: f64 = 0.8; // Synchronization > 0.8 = Seizure
const COUPLING_STRENGTH: f64 = 2.0;
const TIME_STEPS: usize = 20;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Virtual Epilepsy Surgery Planning ===\n");

    // 1. Clinical Input: Connectome (Adjacency List)
    // We mock a small network with a "Hub" (Node 0) driving synchronization (Seizure focus).
    let num_regions = 10;

    // Adjacency Matrix (undirected)
    // Node 0 is connected to everyone (Hub/Focus)
    // Others are sparsely connected
    let mut adj_matrix = vec![vec![]; num_regions];
    for i in 1..num_regions {
        adj_matrix[0].push(i);
        adj_matrix[i].push(0);
        // Add some random connections
        if i < num_regions - 1 {
            adj_matrix[i].push(i + 1);
            adj_matrix[i + 1].push(i);
        }
    }

    println!("[Baseline] Simulating sample brain network...");
    let baseline_graph = build_brain_graph(&adj_matrix, num_regions)?;
    let is_seizing = run_seizure_simulation(&baseline_graph)?;

    if is_seizing {
        println!("-> PATIENT STATUS: SEIZURE DETECTED (High Synchronization)\n");
    } else {
        println!("-> PATIENT STATUS: HEALTHY. No surgery needed.");
        return Ok(());
    }

    println!("--- Starting Virtual Resection Analysis ---");
    println!("Simulating removal of each region to test for seizure freedom...\n");

    // 2. Virtual Resection Loop
    // Iterate through each node, pretend to resect it, check if seizure stops.
    for target_node in 0..num_regions {
        // Construct "Post-Op" Connectome
        // usage: filter out edges involving target_node.
        let post_op_graph = build_brain_graph_resected(&adj_matrix, num_regions, target_node)?;

        let seizure_persists = run_seizure_simulation(&post_op_graph)?;

        if !seizure_persists {
            println!(
                " [SUCCESS] Resection of Region {:>2} is CURATIVE.",
                target_node
            );
            println!(
                "           Recommendation: Target Region {} for ablation.",
                target_node
            );
        } else {
            // println!(" [FAILURE] Resection of Region {:>2} does NOT stop seizures.", target_node);
        }
    }

    Ok(())
}

/// Simulation Kernel: Kuramoto Model
/// Returns true if Seizure (high sync), false otherwise.
fn run_seizure_simulation(graph: &Graph<RegionState>) -> Result<bool, Box<dyn std::error::Error>> {
    let dt = 0.1;

    // Use getter for data (Graph -> CausalTensor), then getter for Vec (CausalTensor -> Vec)
    let phases = Phases(graph.data().data().iter().map(|d| d.phase).collect());
    let freqs: Vec<f64> = graph
        .data()
        .data()
        .iter()
        .map(|d| d.intrinsic_freq)
        .collect();
    let n = phases.0.len();

    // Precompute the adjacency so the rate field below is a pure `Fn(&Phases) -> Phases`.
    let adj: Vec<Vec<usize>> = (0..n)
        .map(|i| graph.neighbors(i).map(|ns| ns.to_vec()))
        .collect::<Result<_, _>>()?;

    // The Kuramoto rate field `dθ_i/dt = ω_i + (K/N)·Σ_j sin(θ_j − θ_i)` as an `Euler` endo-arrow.
    // Swapping `Euler` for `Rk4` would raise the integration order with no change to this rate field.
    let step = Euler::new(dt, move |p: &Phases| {
        let d = (0..n)
            .map(|i| {
                let coupling: f64 = adj[i].iter().map(|&j| (p.0[j] - p.0[i]).sin()).sum();
                freqs[i] + (COUPLING_STRENGTH / n as f64) * coupling
            })
            .collect();
        Phases(d)
    });

    // March `TIME_STEPS` steps; the seizure verdict reads the synchronization of the final state.
    let final_phases = step.iterate_n(phases, TIME_STEPS);

    let sum_cos: f64 = final_phases.0.iter().map(|p| p.cos()).sum();
    let sum_sin: f64 = final_phases.0.iter().map(|p| p.sin()).sum();
    let final_sync = (sum_cos.powi(2) + sum_sin.powi(2)).sqrt() / n as f64;

    Ok(final_sync > SEIZURE_THRESHOLD)
}

/// Kuramoto phase vector — the integrator state. `Euler`/`Rk4` need a module-valued state
/// (`Add` + scalar `Mul`), which `Vec<f64>` lacks, so the phases ride in this newtype.
#[derive(Clone)]
struct Phases(Vec<f64>);

impl Add for Phases {
    type Output = Phases;
    fn add(self, rhs: Phases) -> Phases {
        Phases(self.0.iter().zip(rhs.0).map(|(a, b)| a + b).collect())
    }
}

impl Mul<f64> for Phases {
    type Output = Phases;
    fn mul(self, s: f64) -> Phases {
        Phases(self.0.iter().map(|x| x * s).collect())
    }
}

fn build_brain_graph(
    adj: &[Vec<usize>],
    n: usize,
) -> Result<Graph<RegionState>, Box<dyn std::error::Error>> {
    // Initial random phases
    let mut data = Vec::with_capacity(n);
    for i in 0..n {
        data.push(RegionState {
            phase: (i as f64 * 0.5) % (2.0 * PI),   // Random-ish start
            intrinsic_freq: 1.0 + (i as f64 * 0.1), // Varied frequencies
        });
    }

    let tensor = CausalTensor::new(data, vec![n])?;
    let mut graph = Graph::new(n, tensor, 0)?;

    for (u, neighbors) in adj.iter().enumerate() {
        for &v in neighbors {
            if u < v {
                // Add only once per pair
                graph.add_edge(u, v)?;
            }
        }
    }
    Ok(graph)
}

/// Builds graph excluding a node (Virtual Resection)
fn build_brain_graph_resected(
    adj: &[Vec<usize>],
    n: usize,
    target: usize,
) -> Result<Graph<RegionState>, Box<dyn std::error::Error>> {
    // Initial random phases
    let mut data = Vec::with_capacity(n);
    for i in 0..n {
        data.push(RegionState {
            phase: (i as f64 * 0.5) % (2.0 * PI),
            intrinsic_freq: 1.0 + (i as f64 * 0.1),
        });
    }

    let tensor = CausalTensor::new(data, vec![n])?;
    let mut graph = Graph::new(n, tensor, 0)?;

    for (u, neighbors) in adj.iter().enumerate() {
        // If u is target, we don't add its edges. Simple isolation.
        // It stays in graph as disconnected node.
        if u == target {
            continue;
        }

        for &v in neighbors {
            if v == target {
                continue;
            }
            if u < v {
                graph.add_edge(u, v)?;
            }
        }
    }
    Ok(graph)
}

/// Represents a Brain Region (node)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RegionState {
    pub phase: f64, // Oscillator phase [0, 2pi]
    pub intrinsic_freq: f64,
}

impl Add for RegionState {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            phase: self.phase + rhs.phase,
            intrinsic_freq: self.intrinsic_freq + rhs.intrinsic_freq,
        }
    }
}

impl deep_causality_num::Zero for RegionState {
    fn zero() -> Self {
        Self {
            phase: 0.0,
            intrinsic_freq: 0.0,
        }
    }
    fn is_zero(&self) -> bool {
        self.phase == 0.0 && self.intrinsic_freq == 0.0
    }
}
