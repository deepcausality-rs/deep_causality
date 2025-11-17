/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::model;
use deep_causality::*;

pub fn run_rung2_intervention() {
    println!("--- Rung 2: Intervention ---");
    println!("Demonstrating an intervention: If high cancer risk is detected, prescribe therapy.");

    // 1. Setup Causal Model (same as Rung 1)
    let (graph, smoke_idx, cancer_idx) = model::get_causaloid_graph();

    // The causaloid representing the final effect we want to act upon.
    let final_risk_causaloid = graph.get_causaloid(cancer_idx).unwrap().clone();

    // 2. Define State and Action
    let high_cancer_risk_state = CausalState::new(
        10,                                    // state ID
        1,                                     // version
        PropagatingEffect::from_boolean(true), // The data to evaluate against the causaloid
        final_risk_causaloid,
        None,
    );

    let prescribe_therapy_action = CausalAction::new(
        || {
            println!("Intervention: Cessation therapy prescribed.");
            Ok(())
        },
        "Prescribe Therapy",
        1,
    );

    // 3. Build Causal State Machine (CSM)
    let state_action_pair = &[(&high_cancer_risk_state, &prescribe_therapy_action)];
    let csm = CSM::new(state_action_pair, None);

    // 4. Execute and Intervene
    // We need to evaluate the full causal chain first to get the final effect.
    let initial_effect = PropagatingEffect::from_numerical(0.8);
    let final_effect =
        graph.evaluate_shortest_path_between_causes(smoke_idx, cancer_idx, &initial_effect);
    assert!(final_effect.is_ok());

    // Now, use the final effect as input to the CSM.
    // The CSM will check if the `high_cancer_risk_state` is met by this effect.
    let result = csm.eval_single_state(10, &final_effect);

    assert!(result.is_ok());
    println!("Result: High cancer risk was detected, and the intervention was successfully fired.");
    println!("\n");
}
