/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Quantum causal model — the freeze-time Markov commutativity check.
//!
//! A quantum causal model (Lorenz 2022) is valid only when the Choi–Jamiołkowski
//! factors sharing a Hilbert support pairwise commute. This example carries those
//! factors as an external `ProcessFactors` decoration and runs the check at the
//! freeze boundary via `freeze_quantum`:
//!
//!   * a **commuting** model (two diagonal factors on the shared leg) freezes and
//!     reports the pairwise checks it ran;
//!   * a **non-commuting** model (σx and σz on the shared leg) aborts the freeze,
//!     names the offending operator pair, and rolls the graph back to dynamic.

mod constants;
mod model;

use deep_causality::CausableGraph;
use deep_causality_quantum::{CommutatorTolerance, QuantumErrorEnum, freeze_quantum};

use crate::constants::FloatType;
use crate::model::{diagonal, factors_on_shared_leg, sigma_x, sigma_z, two_node_graph};

fn main() {
    println!("=== Quantum causal model: freeze-time Markov commutativity check ===\n");

    commuting_model_freezes();
    println!();
    non_commuting_model_aborts();
}

/// Two diagonal factors on the shared leg commute → the model freezes.
fn commuting_model_freezes() {
    println!("[1] Commuting model: σz and diag(3, -1) on leg 0");

    let mut graph = two_node_graph();
    let (factors, supports) = factors_on_shared_leg(sigma_z(), diagonal(3.0, -1.0));
    let tolerance = CommutatorTolerance::<FloatType>::default();

    match freeze_quantum(&mut graph, &[], &factors, &supports, &tolerance) {
        Ok(report) => {
            println!(
                "    ✓ froze cleanly; {} commuting pair(s) checked, worst margin {:.3e}",
                report.tested_pairs(),
                report.worst_margin().unwrap_or(0.0)
            );
            println!("    is_frozen() = {}", graph.is_frozen());
        }
        Err(e) => println!("    unexpected freeze failure: {}", e),
    }
}

/// σx and σz on the shared leg do not commute → the freeze aborts.
fn non_commuting_model_aborts() {
    println!("[2] Non-commuting model: σx and σz on leg 0");

    let mut graph = two_node_graph();
    let (factors, supports) = factors_on_shared_leg(sigma_x(), sigma_z());
    let tolerance = CommutatorTolerance::<FloatType>::default();

    match freeze_quantum(&mut graph, &[], &factors, &supports, &tolerance) {
        Ok(_) => println!("    unexpected: a non-commuting model should not freeze"),
        Err(e) => match e.0 {
            QuantumErrorEnum::CommutatorNonZero { node_j, node_k, .. } => {
                println!(
                    "    ✓ freeze aborted: factors at nodes {} and {} do not commute",
                    node_j, node_k
                );
                println!("    is_frozen() = {} (rolled back)", graph.is_frozen());
            }
            other => println!("    aborted with an unexpected error: {:?}", other),
        },
    }
}
