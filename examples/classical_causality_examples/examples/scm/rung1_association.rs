/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::model::ScmState;
use deep_causality::*;

pub fn run_rung1_association(_explain: bool) {
    println!("--- Rung 1: Association ---");
    println!("Demonstrating observational inference: Given smoking, what is the cancer risk?");

    // 1. Build CausaloidGraph
    let (graph, smoke_idx, cancer_idx) = crate::model::get_causaloid_graph();

    // 2. Execute and Observe
    // Represents observing a high nicotine level.
    let initial_state = ScmState {
        nicotine_level: 0.8, // High nicotine
        has_high_nicotine: false,
        has_tar: false,
        cancer_risk: false,
    };
    let initial_effect: PropagatingEffect<ScmState> = PropagatingEffect::pure(initial_state);

    // Evaluate the full chain of events starting from the first cause.
    let final_effect =
        graph.evaluate_shortest_path_between_causes(smoke_idx, cancer_idx, &initial_effect);

    if final_effect.is_err() {
        eprintln!("Evaluation failed: {:?}", final_effect.error);
        return;
    }

    // 3. Assert and Explain
    let result = final_effect.value.into_value().unwrap_or(initial_state);
    assert!(result.cancer_risk, "Expected high cancer risk");
    println!("Result: Observation of high nicotine level is associated with high cancer risk.");
    println!("\n");
}
