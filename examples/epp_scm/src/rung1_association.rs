/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;

pub fn run_rung1_association(explain: bool) {
    println!("--- Rung 1: Association ---");
    println!("Demonstrating observational inference: Given smoking, what is the cancer risk?");

    // 1. Build CausaloidGraph
    let (graph, smoke_idx, cancer_idx) = crate::model::get_causaloid_graph();

    // 2. Execute and Observe
    // Represents observing a high nicotine level.
    let initial_effect = PropagatingEffect::from_numerical(0.8);

    // Evaluate the full chain of events starting from the first cause.
    let final_effect =
        graph.evaluate_shortest_path_between_causes(smoke_idx, cancer_idx, &initial_effect);
    assert!(final_effect.is_ok(), "{:?}", final_effect);

    // 3. Assert and Explain
    assert_eq!(final_effect.value, EffectValue::Boolean(true));
    println!("Result: Observation of high nicotine level is associated with high cancer risk.");
    println!("\n");

    if explain {
        // Explain the line of reasoning:
        println!("Explain:");
        println!("{}", final_effect.explain());
        println!("\n");
    }
}
