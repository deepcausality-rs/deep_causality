/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # A quantum causal model, and the check that decides whether it is one
//!
//! A quantum causal model describes a process as a graph whose nodes carry Choi–Jamiołkowski
//! factors. Not every such graph is a valid model. The factors that share a Hilbert leg have to
//! **pairwise commute**, because the model's Markov condition is a statement about them being
//! simultaneously assignable, and non-commuting operators have no joint assignment to make.
//!
//! So the condition is not a modelling preference. A graph that fails it is not a quantum causal
//! model that happens to be awkward; it is not a quantum causal model.
//!
//! # Freezing is where the check belongs
//!
//! A causal graph in this library is **dynamic** while it is being built and **frozen** once it is
//! ready to run. Freezing is the one moment at which the structure is complete and nothing has yet
//! depended on it, which makes it the only place a structural condition can be enforced without
//! either rejecting a half-built graph or discovering the problem after a result has been used.
//!
//! `freeze_quantum` runs the pairwise commutator checks at that boundary. On success the graph
//! freezes and the report says which pairs were tested and by how much they passed. On failure the
//! freeze **aborts and rolls the graph back to dynamic**, so a model that cannot be frozen is never
//! left half-frozen.
//!
//! # What the run does
//!
//! Two models, differing only in one operator:
//!
//! ```text
//! sigma_z and diag(3, -1) on leg 0   commute        freezes, reports the margin
//! sigma_x and sigma_z     on leg 0   anticommute    aborts, names the pair, rolls back
//! ```
//!
//! Pauli X and Z anticommute, so their commutator is `2i σ_y` and its norm is as far from zero as a
//! single-qubit commutator gets. The two diagonal operators commute exactly, and the margin the
//! report carries says so.

mod constants;
mod model;
mod utils_print;

use deep_causality::CausableGraph;
use deep_causality_num::Float106;
use deep_causality_quantum::{CommutatorTolerance, QuantumErrorEnum, freeze_quantum};
use model::{diagonal, factors_on_shared_leg, sigma_x, sigma_z, two_node_graph};
use utils_print::{print_abort, print_frozen, print_header, print_scenario};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the operators,
/// the commutator norms and the tolerance all recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    commuting_model_freezes()?;
    println!();
    non_commuting_model_aborts()?;

    Ok(())
}

/// Two commuting factors on the shared leg, so the model freezes.
fn commuting_model_freezes() -> Result<(), Box<dyn std::error::Error>> {
    print_scenario(1, "sigma_z and diag(3, -1) on leg 0", "they commute");

    let mut graph = two_node_graph()?;
    let (factors, supports) = factors_on_shared_leg(sigma_z()?, diagonal()?);
    let tolerance = CommutatorTolerance::<FloatType>::default();

    match freeze_quantum(&mut graph, &[], &factors, &supports, &tolerance, None) {
        Ok(report) => print_frozen(&report, graph.is_frozen()),
        Err(e) => return Err(format!("a commuting model failed to freeze: {e}").into()),
    }

    Ok(())
}

/// Two anticommuting factors on the shared leg, so the freeze aborts and the graph rolls back.
fn non_commuting_model_aborts() -> Result<(), Box<dyn std::error::Error>> {
    print_scenario(2, "sigma_x and sigma_z on leg 0", "they anticommute");

    let mut graph = two_node_graph()?;
    let (factors, supports) = factors_on_shared_leg(sigma_x()?, sigma_z()?);
    let tolerance = CommutatorTolerance::<FloatType>::default();

    match freeze_quantum(&mut graph, &[], &factors, &supports, &tolerance, None) {
        Ok(_) => return Err("a non-commuting model froze; the freeze should abort".into()),
        Err(e) => match e.0 {
            QuantumErrorEnum::CommutatorNonZero {
                node_j,
                node_k,
                detail,
            } => print_abort(node_j, node_k, &detail, graph.is_frozen()),
            other => {
                return Err(format!("the freeze aborted for the wrong reason: {other:?}").into());
            }
        },
    }

    Ok(())
}
