/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Virtual Resection as `do(connectome = resected_at_R)`
//!
//! Counterfactual interventions on the connectome carried by the chain.
//! The factual run simulates seizure dynamics on the full patient
//! connectome; synchronization rises above the seizure threshold.
//! Each candidate resection is one intervention on the same chain: the
//! connectome value is replaced with a copy of itself in which one
//! region has been disconnected from its neighbours. The downstream
//! Kuramoto simulation then evaluates post-resection synchronization
//! on the same dynamics.
//!
//! ## Why this pattern matters clinically
//!
//! In a surgical-planning workflow the artefact that matters is not the
//! simulator's final number; it is the audit trail recording which
//! region was virtually resected and what the model predicted for that
//! resection. `EffectLog` records every `!!ValueAlternation!!` entry
//! automatically. A plain re-run loop computes the same numbers and
//! loses the link between *which* resection produced *which* outcome.
//! That link has to be reconstructed externally if the monad does not
//! preserve it for you.
//!
//! The chain is the same for every candidate. Only the connectome value
//! at the moment of intervention changes. Patient-specific dynamics
//! (intrinsic frequencies, initial phases, coupling strength) stay
//! constant across the counterfactual sweep.

mod model;
pub mod model_types;
mod model_utils;

use deep_causality_core::{CausalFlow, PropagatingEffect};
use model::simulate_seizure;
use model_types::{Connectome, N_REGIONS, SeizureResult};

fn main() {
    println!("=== Virtual Resection as `do(connectome = resected_at_R)` ===\n");

    let factual = Connectome::hub_and_spoke(N_REGIONS);
    model_utils::print_connectome_header();

    let f = run_factual(factual.clone());
    model_utils::print_process("Factual (no resection)", &f, None);

    model_utils::print_resection_screening(&factual, run_counterfactual);
    model_utils::print_audit_trail(&factual, run_counterfactual);
}

fn run_factual(connectome: Connectome) -> PropagatingEffect<SeizureResult> {
    CausalFlow::value(connectome)
        .map(simulate_seizure)
        .into_effect()
}

fn run_counterfactual(
    factual: Connectome,
    resected: Connectome,
) -> PropagatingEffect<SeizureResult> {
    CausalFlow::value(factual)
        .alternate_value(resected)
        .map(simulate_seizure)
        .into_effect()
}
