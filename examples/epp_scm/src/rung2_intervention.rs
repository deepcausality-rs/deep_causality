/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;

fn get_smoking_causaloid() -> BaseCausaloid {
    Causaloid::new(1, |effect| {
        let nicotine_level = effect.as_numerical().unwrap_or(0.0);
        Ok(PropagatingEffect::Deterministic(nicotine_level > 0.6))
    }, "Smoking Status")
}

fn get_tar_causaloid() -> BaseCausaloid {
    Causaloid::new(2, |effect| {
        let is_smoking = effect.as_bool().unwrap_or(false);
        Ok(PropagatingEffect::Deterministic(is_smoking))
    }, "Tar in Lungs")
}

fn get_cancer_risk_causaloid() -> BaseCausaloid {
    Causaloid::new(3, |effect| {
        let has_tar = effect.as_bool().unwrap_or(false);
        Ok(PropagatingEffect::Deterministic(has_tar))
    }, "Cancer Risk")
}

pub fn run_rung2_intervention() {
    println!("--- Rung 2: Intervention ---");
    println!("Demonstrating an intervention: If high cancer risk is detected, prescribe therapy.");

    // 1. Setup Causal Model (same as Rung 1)
    let mut graph = CausaloidGraph::new(1);
    let smoke_idx = graph.add_causaloid(get_smoking_causaloid()).unwrap();
    let tar_idx = graph.add_causaloid(get_tar_causaloid()).unwrap();
    let cancer_idx = graph.add_causaloid(get_cancer_risk_causaloid()).unwrap();
    graph.add_edge(smoke_idx, tar_idx).unwrap();
    graph.add_edge(tar_idx, cancer_idx).unwrap();
    graph.freeze();

    // The causaloid representing the final effect we want to act upon.
    let final_risk_causaloid = graph.get_causaloid(cancer_idx).unwrap().clone();

    // 2. Define State and Action
    let high_cancer_risk_state = CausalState::new(
        10, // state ID
        1,  // version
        PropagatingEffect::Deterministic(true), // The data to evaluate against the causaloid
        final_risk_causaloid,
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
    let csm = CSM::new(state_action_pair);

    // 4. Execute and Intervene
    // We need to evaluate the full causal chain first to get the final effect.
    let initial_effect = PropagatingEffect::Numerical(0.8);
    let final_effect = graph.evaluate_shortest_path_between_causes(smoke_idx, cancer_idx, &initial_effect).unwrap();

    // Now, use the final effect as input to the CSM.
    // The CSM will check if the `high_cancer_risk_state` is met by this effect.
    let result = csm.eval_single_state(10, &final_effect);

    assert!(result.is_ok());
    println!("Result: High cancer risk was detected, and the intervention was successfully fired.");
    println!("\n");
}
