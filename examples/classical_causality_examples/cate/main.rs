/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use std::sync::{Arc, RwLock};

mod model;

// Define IDs for different data types within the context
const AGE_ID: IdentificationValue = 1;
const INITIAL_BP_ID: IdentificationValue = 2;
const DRUG_ADMINISTERED_ID: IdentificationValue = 3;

// Define ID for the causaloid
const DRUG_EFFECT_CAUSALOID_ID: IdentificationValue = 10;

fn main() {
    println!("\n--- CATE Example: Effect of Medication on Blood Pressure for Patients > 65 ---");

    // 1. Define the population of patients
    let patient_population = model::create_patient_population();
    println!(
        "Created a population of {} patients.",
        patient_population.len()
    );

    // 2. Select the subgroup of interest (patients over 65)
    let subgroup: Vec<&BaseContext> = patient_population
        .iter()
        .filter(|ctx| {
            for i in 0..ctx.number_of_nodes() {
                if let Some(node) = ctx.get_node(i)
                    && let ContextoidType::Datoid(data_node) = node.vertex_type()
                    && data_node.id() == AGE_ID
                    && data_node.get_data() > 65.0
                {
                    return true;
                }
            }
            false
        })
        .collect();
    println!(
        "Found {} patients in the subgroup (age > 65).",
        subgroup.len()
    );

    // 3. Run parallel counterfactuals for the subgroup
    let mut ites: Vec<f64> = Vec::new(); // To store Individual Treatment Effects

    for patient_context in subgroup {
        let initial_bp = model::get_patient_bp(patient_context).unwrap_or(140.0);

        // --- Create Counterfactual Contexts ---
        let mut treatment_context = patient_context.clone();
        let drug_datoid = Contextoid::new(
            DRUG_ADMINISTERED_ID,
            ContextoidType::Datoid(Data::new(DRUG_ADMINISTERED_ID, 1.0)), // drug_administered = true
        );
        treatment_context.add_node(drug_datoid).unwrap();

        let mut control_context = patient_context.clone();
        let no_drug_datoid = Contextoid::new(
            DRUG_ADMINISTERED_ID,
            ContextoidType::Datoid(Data::new(DRUG_ADMINISTERED_ID, 0.0)), // drug_administered = false
        );
        control_context.add_node(no_drug_datoid).unwrap();

        // --- Instantiate Causaloids for each scenario ---
        // New API: ContextualCausalFn = fn(EffectValue<I>, S, Option<C>) -> PropagatingProcess<O, S, C>
        let treatment_causaloid = Causaloid::new_with_context(
            DRUG_EFFECT_CAUSALOID_ID,
            model::drug_effect_logic,
            Arc::new(RwLock::new(treatment_context)),
            "Drug effect under treatment",
        );

        let control_causaloid = Causaloid::new_with_context(
            DRUG_EFFECT_CAUSALOID_ID,
            model::drug_effect_logic,
            Arc::new(RwLock::new(control_context)),
            "Drug effect under control",
        );

        // --- Evaluate Potential Outcomes ---
        // The input effect is the patient's initial BP.
        let input_effect: PropagatingEffect<NumericalValue> = PropagatingEffect::pure(initial_bp);

        let y1_res = treatment_causaloid.evaluate(&input_effect);
        if y1_res.is_err() {
            eprintln!("Treatment evaluation failed: {:?}", y1_res.error);
            continue;
        }

        let y1_effect = y1_res.value.into_value().unwrap_or(0.0);

        let y0_res = control_causaloid.evaluate(&input_effect);
        if y0_res.is_err() {
            eprintln!("Control evaluation failed: {:?}", y0_res.error);
            continue;
        }

        let y0_effect = y0_res.value.into_value().unwrap_or(0.0);

        let y1 = initial_bp + y1_effect; // Potential outcome if treated
        let y0 = initial_bp + y0_effect; // Potential outcome if not treated

        // --- Calculate and Store ITE ---
        let ite = y1 - y0;
        ites.push(ite);
    }

    // 4. Aggregate and Conclude
    if !ites.is_empty() {
        let cate: f64 = ites.iter().sum::<f64>() / ites.len() as f64;
        println!("\n--- CATE Calculation Result ---");
        println!(
            "The Conditional Average Treatment Effect (CATE) for patients over 65 is: {:.2}",
            cate
        );
    } else {
        println!("\nNo patients found in the subgroup to calculate CATE.");
    }
}
