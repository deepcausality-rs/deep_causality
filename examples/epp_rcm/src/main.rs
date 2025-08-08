/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use std::sync::Arc;

const INITIAL_BP_ID: IdentificationValue = 101;
const DRUG_ADMINISTERED_ID: IdentificationValue = 100;

fn main() {
    println!("\n--- RCM Example: Drug Effect on Blood Pressure ---");
    let drug_effect_causaloid = get_drug_effect_causaloid();
    // final_bp_causaloid
    let final_bp_causaloid = get_final_bp_causaloid();

    // Create the CausaloidGraph
    let mut causaloid_graph = CausaloidGraph::new(0);
    let drug_effect_idx = causaloid_graph
        .add_causaloid(drug_effect_causaloid)
        .unwrap();
    let final_bp_idx = causaloid_graph.add_causaloid(final_bp_causaloid).unwrap();
    causaloid_graph
        .add_edge(drug_effect_idx, final_bp_idx)
        .unwrap();
    causaloid_graph.freeze();
    let causaloid_graph_arc = Arc::new(causaloid_graph);

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
    let mut treated_effect_map = PropagatingEffect::new_map();
    treated_effect_map.insert(
        INITIAL_BP_ID,
        PropagatingEffect::Numerical(patient_initial_bp),
    );
    treated_effect_map.insert(DRUG_ADMINISTERED_ID, PropagatingEffect::Numerical(1.0)); // Pass 1.0 for true

    let y1_res =
        causaloid_graph_arc.evaluate_subgraph_from_cause(drug_effect_idx, &treated_effect_map);
    let y1 = y1_res.unwrap().as_numerical().unwrap();
    println!("Y(1) (Treated Outcome): {y1:.1}");

    println!("\nSimulating Control Outcome (Y(0))...");
    let mut control_effect_map = PropagatingEffect::new_map();
    control_effect_map.insert(
        INITIAL_BP_ID,
        PropagatingEffect::Numerical(patient_initial_bp),
    );
    control_effect_map.insert(DRUG_ADMINISTERED_ID, PropagatingEffect::Numerical(0.0)); // Pass 0.0 for false

    let y0_res =
        causaloid_graph_arc.evaluate_subgraph_from_cause(drug_effect_idx, &control_effect_map);
    let y0 = y0_res.unwrap().as_numerical().unwrap();
    println!("Y(0) (Control Outcome): {y0:.1}");

    // 5. Calculate and Report the Causal Effect
    let ite = y1 - y0;
    println!("\nIndividual Treatment Effect (ITE): {ite:.1}");

    println!("\n--- Conclusion ---");
    println!("The Individual Treatment Effect (ITE) for this patient is {ite:.1}.");
    println!(
        "The drug is predicted to lower their blood pressure by {} points.",
        -ite
    );
}

fn get_drug_effect_causaloid() -> BaseCausaloid {
    let drug_effect_causaloid_id = 1;
    let drug_effect_causaloid_description = "Determines drug effect based on administration";

    let drug_effect_causaloid_fn =
        |effect: &PropagatingEffect| -> Result<PropagatingEffect, CausalityError> {
            let drug_effect_causaloid_id = 1;

            let (is_drug_administered, initial_bp) = match effect {
                PropagatingEffect::Map(map) => {
                    let is_drug_administered = map
                        .get(&DRUG_ADMINISTERED_ID)
                        .and_then(|boxed_effect| boxed_effect.as_numerical())
                        .map(|val| val == 1.0) // Interpret 1.0 as true, 0.0 as false
                        .unwrap_or(false); // Default to false if not found or not numerical

                    let initial_bp = map
                        .get(&INITIAL_BP_ID)
                        .and_then(|boxed_effect| boxed_effect.as_numerical())
                        .ok_or_else(|| {
                            CausalityError(
                                "Initial BP not found in effect map for drug_effect_causaloid"
                                    .into(),
                            )
                        })?;
                    (is_drug_administered, initial_bp)
                }
                _ => {
                    return Err(CausalityError(
                        "Expected Map effect for drug_effect_causaloid".into(),
                    ));
                }
            };

            let drug_effect_value = if is_drug_administered { -10.0 } else { 0.0 };

            let mut result_map = PropagatingEffect::new_map();
            result_map.insert(INITIAL_BP_ID, PropagatingEffect::Numerical(initial_bp));
            result_map.insert(
                DRUG_ADMINISTERED_ID,
                PropagatingEffect::Numerical(is_drug_administered as u64 as f64),
            ); // Store as numerical for consistency
            result_map.insert(
                drug_effect_causaloid_id,
                PropagatingEffect::Numerical(drug_effect_value),
            ); // Store the calculated drug effect

            Ok(result_map)
        };
    BaseCausaloid::new(
        drug_effect_causaloid_id,
        drug_effect_causaloid_fn,
        drug_effect_causaloid_description,
    )
}

fn get_final_bp_causaloid() -> BaseCausaloid {
    let final_bp_causaloid_id = 2;
    let final_bp_causaloid_description = "Calculates final blood pressure";
    let final_bp_causaloid_fn =
        |effect: &PropagatingEffect| -> Result<PropagatingEffect, CausalityError> {
            let drug_effect_causaloid_id = 1;

            let (initial_bp, drug_effect) = match effect {
                PropagatingEffect::Map(map) => {
                    let initial_bp = map
                        .get(&INITIAL_BP_ID)
                        .and_then(|boxed_effect| boxed_effect.as_numerical())
                        .ok_or_else(|| {
                            CausalityError(
                                "Initial BP not found in effect map for final_bp_causaloid".into(),
                            )
                        })?;
                    let drug_effect = map
                        .get(&drug_effect_causaloid_id)
                        .and_then(|boxed_effect| boxed_effect.as_numerical())
                        .ok_or_else(|| {
                            CausalityError(
                                "Drug effect value not found in effect map for final_bp_causaloid"
                                    .into(),
                            )
                        })?;
                    (initial_bp, drug_effect)
                }
                _ => {
                    return Err(CausalityError(
                        "Expected Map effect for final BP calculation".into(),
                    ));
                }
            };

            Ok(PropagatingEffect::Numerical(initial_bp + drug_effect))
        };
    BaseCausaloid::new(
        final_bp_causaloid_id,
        final_bp_causaloid_fn,
        final_bp_causaloid_description,
    )
}
