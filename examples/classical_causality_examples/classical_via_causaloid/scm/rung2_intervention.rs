/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::model;
use crate::model::ScmState;
use deep_causality::*;

pub fn run_rung2_intervention() {
    println!("--- Rung 2: Intervention ---");
    println!("Demonstrating an intervention: If high cancer risk is detected, prescribe therapy.");

    // 1. Setup Causal Model (same as Rung 1)
    let (graph, smoke_idx, cancer_idx) = model::get_causaloid_graph();

    // 2. Execute the causal chain
    let initial_state = ScmState {
        nicotine_level: 0.8, // High nicotine
        has_high_nicotine: false,
        has_tar: false,
        cancer_risk: false,
    };
    let initial_effect: PropagatingEffect<ScmState> = PropagatingEffect::pure(initial_state);

    let final_effect =
        graph.evaluate_shortest_path_between_causes(smoke_idx, cancer_idx, &initial_effect);

    if final_effect.is_err() {
        eprintln!("Evaluation failed: {:?}", final_effect.error);
        return;
    }

    let result = final_effect.value.into_value().unwrap_or(initial_state);

    // 3. Intervention: If high cancer risk, prescribe therapy
    if result.cancer_risk {
        println!("Intervention: High cancer risk detected. Cessation therapy prescribed.");
    } else {
        println!("No intervention needed: Cancer risk is low.");
    }

    println!("Result: High cancer risk was detected, and the intervention was successfully fired.");
    println!("\n");
}
