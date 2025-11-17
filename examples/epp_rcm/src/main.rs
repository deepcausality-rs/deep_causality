/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod model;
mod types;

use deep_causality::*;
use std::collections::HashMap;
use std::sync::Arc;

const INITIAL_BP_ID: IdentificationValue = 101;
const DRUG_ADMINISTERED_ID: IdentificationValue = 100;

fn main() {
    println!("\n--- RCM Example: Drug Effect on Blood Pressure ---");
    let drug_effect_idx = 0;
    // build the Model and wrap into into an arc
    let causaloid_graph_arc = Arc::new(model::get_causaloid_graph());

    // 2. Define the Unit's Baseline State (The Patient)
    let patient_initial_bp = 145.0;
    let patient_baseline_context_id = 1;
    let patient_baseline_context_name = "Patient Baseline";
    let mut patient_baseline_context = BaseContext::with_capacity(
        patient_baseline_context_id,
        patient_baseline_context_name,
        10,
    );
    let initial_bp_datoid = Contextoid::new(
        INITIAL_BP_ID, // ID for initial_blood_pressure
        ContextoidType::Datoid(Data::new(INITIAL_BP_ID, patient_initial_bp)),
    );
    patient_baseline_context
        .add_node(initial_bp_datoid)
        .unwrap();
    let patient_baseline_context_arc = Arc::new(patient_baseline_context);

    // 3. Create the Potential Worlds (Contextual Alternation)
    // Treatment Context
    let mut treatment_context = (*patient_baseline_context_arc).clone();
    let drug_administered_datoid_true = Contextoid::new(
        DRUG_ADMINISTERED_ID, // ID for drug_administered
        ContextoidType::Datoid(Data::new(DRUG_ADMINISTERED_ID, 1.0)), // Use 1.0 for true
    );
    treatment_context
        .add_node(drug_administered_datoid_true)
        .unwrap();

    // Control Context
    let mut control_context = (*patient_baseline_context_arc).clone();
    let drug_administered_datoid_false = Contextoid::new(
        DRUG_ADMINISTERED_ID, // ID for drug_administered
        ContextoidType::Datoid(Data::new(DRUG_ADMINISTERED_ID, 0.0)), // Use 0.0 for false
    );
    control_context
        .add_node(drug_administered_datoid_false)
        .unwrap();

    // 4. Simulate Both Potential Outcomes
    println!("\nSimulating Treated Outcome (Y(1))...");
    let mut map = HashMap::new();
    map.insert(
        INITIAL_BP_ID,
        Box::new(PropagatingEffect::from_numerical(patient_initial_bp)),
    );
    map.insert(
        DRUG_ADMINISTERED_ID,
        Box::new(PropagatingEffect::from_numerical(1.0)),
    ); // Pass 1.0 for true

    let treated_effect_map = PropagatingEffect::from_map(map);
    let y1_res =
        causaloid_graph_arc.evaluate_subgraph_from_cause(drug_effect_idx, &treated_effect_map);
    assert!(y1_res.is_ok());

    println!(
        "Y(1) (Treated Outcome): {}",
        &y1_res.value.as_numerical().unwrap()
    );

    println!("\nSimulating Control Outcome (Y(0))...");
    let mut map = HashMap::new();

    map.insert(
        INITIAL_BP_ID,
        Box::new(PropagatingEffect::from_numerical(patient_initial_bp)),
    );
    map.insert(
        DRUG_ADMINISTERED_ID,
        Box::new(PropagatingEffect::from_numerical(0.0)),
    ); // Pass 0.0 for false

    let y0_res = causaloid_graph_arc
        .evaluate_subgraph_from_cause(drug_effect_idx, &PropagatingEffect::from_map(map));
    println!(
        "Y(0) (Control Outcome): {}",
        &y0_res.value.as_numerical().unwrap()
    );

    // 5. Calculate and Report the Causal Effect
    let y0 = y0_res.value.as_numerical().unwrap();
    let y1 = y1_res.value.as_numerical().unwrap();
    let ite = y1 - y0;
    println!("\nIndividual Treatment Effect (ITE): {ite:.1}");

    println!("\n--- Conclusion ---");
    println!("The Individual Treatment Effect (ITE) for this patient is {ite:.1}.");
    println!(
        "The drug is predicted to lower their blood pressure by {} points.",
        -ite
    );
}
