/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod model;

use crate::model::RcmState;
use deep_causality::*;
use std::sync::Arc;

fn main() {
    println!("\n--- RCM Example: Drug Effect on Blood Pressure ---");
    let drug_effect_idx = 0;
    // Build the Model and wrap into an Arc
    let causaloid_graph_arc = Arc::new(model::get_causaloid_graph());

    // 2. Define the Unit's Baseline State (The Patient)
    let patient_initial_bp = 145.0;

    // 3. Create the Potential Worlds (Contextual Alternation)
    // Treatment State
    let treatment_state = RcmState {
        initial_bp: patient_initial_bp,
        drug_administered: true,
        drug_effect: 0.0,
        final_bp: 0.0,
    };

    // Control State
    let control_state = RcmState {
        initial_bp: patient_initial_bp,
        drug_administered: false,
        drug_effect: 0.0,
        final_bp: 0.0,
    };

    // 4. Simulate Both Potential Outcomes
    println!("\nSimulating Treated Outcome (Y(1))...");
    let treated_effect: PropagatingEffect<RcmState> = PropagatingEffect::pure(treatment_state);
    let y1_res = causaloid_graph_arc.evaluate_subgraph_from_cause(drug_effect_idx, &treated_effect);

    if y1_res.is_err() {
        eprintln!("Treatment evaluation failed: {:?}", y1_res.error);
        return;
    }

    let y1_state = y1_res.value.into_value().unwrap_or(treatment_state);
    println!("Y(1) (Treated Outcome): {:.1}", y1_state.final_bp);

    println!("\nSimulating Control Outcome (Y(0))...");
    let control_effect: PropagatingEffect<RcmState> = PropagatingEffect::pure(control_state);
    let y0_res = causaloid_graph_arc.evaluate_subgraph_from_cause(drug_effect_idx, &control_effect);

    if y0_res.is_err() {
        eprintln!("Control evaluation failed: {:?}", y0_res.error);
        return;
    }

    let y0_state = y0_res.value.into_value().unwrap_or(control_state);
    println!("Y(0) (Control Outcome): {:.1}", y0_state.final_bp);

    // 5. Calculate and Report the Causal Effect
    let ite = y1_state.final_bp - y0_state.final_bp;
    println!("\nIndividual Treatment Effect (ITE): {ite:.1}");

    println!("\n--- Conclusion ---");
    println!("The Individual Treatment Effect (ITE) for this patient is {ite:.1}.");
    println!(
        "The drug is predicted to lower their blood pressure by {} points.",
        -ite
    );
}
