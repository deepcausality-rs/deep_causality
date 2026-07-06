/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Counterfactual Cascading Failure (Flow Network)
//!
//! A small fluid distribution network, modelled as a stateful
//! `PropagatingProcess`. The factual chain solves the baseline flows. A
//! triggering pipe failure goes in as `.intervene(failure)`. From there a
//! cascade loop iterates: re-solve the flow, find pipes that now exceed
//! their capacity, and apply the next intervention onto the chain that
//! already carries the previous intervention. The loop stops when a
//! re-solve finds no overloads (stable footprint) or when an iteration
//! cap hits (system collapse).
//!
//! In power-grid planning this is called N-k contingency analysis. Here
//! it is one chain of `.intervene` calls.
//!
//! ## What this gets you that a plain re-solve loop does not
//!
//! 1. Interventions compose. Each cascade step is `intervene` applied to
//!    the chain that already carries the previous one. There is one
//!    chain, not N separate computations.
//! 2. The `EffectLog` is the forensic timeline. After a five-step
//!    cascade, the log reads as a play-by-play: which pipe failed when,
//!    and what triggered the next failure. That is the artifact a
//!    planner actually wants.
//! 3. State persists across interventions. `NetworkState` accumulates
//!    `failed_edges` across the cascade; intervening on the value channel
//!    does not reset that history.
//! 4. The fixed point is the answer. "What if pipe X fails?" gets
//!    resolved by running the cascade and reading off the final
//!    footprint; that footprint is the counterfactual quantity of
//!    interest.

mod model;
pub mod model_config;
pub mod model_types;
mod model_utils;

use deep_causality_core::{EffectLog, EffectValue};
use model::{resolve_stage, run_cascade};
use model_config::build_network;
use model_types::{FlowSolution, NetworkConfig, NetworkProcess, NetworkState};

fn main() {
    println!("=== Counterfactual Cascading Failure (Flow Network) ===\n");

    let cfg = build_network();
    model_utils::print_network_topology(&cfg);

    let factual = run_factual(cfg.clone());
    let cascade_a = run_cascade(0, cfg.clone());
    let cascade_b = run_cascade(1, cfg);

    model_utils::print_section("Factual flows (no failures)", &factual);
    model_utils::print_section("Counterfactual A: do(edge 0 fails)", &cascade_a);
    model_utils::print_section("Counterfactual B: do(edge 1 fails)", &cascade_b);

    println!("=== Summary ===");
    model_utils::summary_line("Factual          ", &factual);
    model_utils::summary_line("do(edge 0 fails) ", &cascade_a);
    model_utils::summary_line("do(edge 1 fails) ", &cascade_b);
}

fn run_factual(cfg: NetworkConfig) -> NetworkProcess<FlowSolution> {
    NetworkProcess::<Vec<u32>>::new(
        Ok(EffectValue::Value(Vec::new())),
        NetworkState::default(),
        Some(cfg),
        EffectLog::new(),
    )
    .bind(resolve_stage)
}
