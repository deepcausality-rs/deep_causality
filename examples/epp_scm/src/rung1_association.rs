/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;

// Define Causaloids
fn get_smoking_causaloid() -> BaseCausaloid {
    Causaloid::new(1, |effect| {
        let nicotine_level = effect.as_numerical().unwrap_or(0.0);
        Ok(PropagatingEffect::Deterministic(nicotine_level > 0.6))
    }, "Smoking Status")
}

fn get_tar_causaloid() -> BaseCausaloid {
    Causaloid::new(2, |effect| {
        // Tar is present if the preceding cause (smoking) is active.
        let is_smoking = effect.as_bool().unwrap_or(false);
        Ok(PropagatingEffect::Deterministic(is_smoking))
    }, "Tar in Lungs")
}

fn get_cancer_risk_causaloid() -> BaseCausaloid {
    Causaloid::new(3, |effect| {
        // Cancer risk is high if tar is present.
        let has_tar = effect.as_bool().unwrap_or(false);
        Ok(PropagatingEffect::Deterministic(has_tar))
    }, "Cancer Risk")
}

pub fn run_rung1_association() {
    println!("--- Rung 1: Association ---");
    println!("Demonstrating observational inference: Given smoking, what is the cancer risk?");

    // 1. Build CausaloidGraph
    let mut graph = CausaloidGraph::new(1);
    let smoke_idx = graph.add_causaloid(get_smoking_causaloid()).unwrap();
    let tar_idx = graph.add_causaloid(get_tar_causaloid()).unwrap();
    let cancer_idx = graph.add_causaloid(get_cancer_risk_causaloid()).unwrap();

    graph.add_edge(smoke_idx, tar_idx).unwrap();
    graph.add_edge(tar_idx, cancer_idx).unwrap();
    graph.freeze();

    // 2. Execute and Observe
    // Represents observing a high nicotine level.
    let initial_effect = PropagatingEffect::Numerical(0.8);
    
    // Evaluate the full chain of events starting from the first cause.
    let final_effect = graph.evaluate_shortest_path_between_causes(smoke_idx, cancer_idx, &initial_effect).unwrap();

    // 3. Assert and Explain
    assert_eq!(final_effect, PropagatingEffect::Deterministic(true));
    println!("Result: Observation of high nicotine level is associated with high cancer risk.");
    println!("\n");
}
