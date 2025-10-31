/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use std::sync::{Arc, RwLock};

// Define IDs for different data types within the context
const AGE_ID: IdentificationValue = 1;
const INITIAL_BP_ID: IdentificationValue = 2;
const DRUG_ADMINISTERED_ID: IdentificationValue = 3;

// Define ID for the causaloid
const DRUG_EFFECT_CAUSALOID_ID: IdentificationValue = 10;

fn main() {
    println!("\n--- CATE Example: Effect of Medication on Blood Pressure for Patients > 65 ---");

    // 1. Define the population of patients
    let patient_population = create_patient_population();
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
        let initial_bp = get_patient_bp(patient_context).unwrap_or(140.0);

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
        let treatment_causaloid = Causaloid::new_with_context(
            DRUG_EFFECT_CAUSALOID_ID,
            drug_effect_logic,
            Arc::new(RwLock::new(treatment_context)),
            "Drug effect under treatment",
        );

        let control_causaloid = Causaloid::new_with_context(
            DRUG_EFFECT_CAUSALOID_ID,
            drug_effect_logic,
            Arc::new(RwLock::new(control_context)),
            "Drug effect under control",
        );

        // --- Evaluate Potential Outcomes ---
        // The input effect is the patient's initial BP.
        let input_effect = PropagatingEffect::Numerical(initial_bp);

        let y1_effect = treatment_causaloid
            .evaluate(&input_effect)
            .unwrap()
            .as_numerical()
            .unwrap();
        let y0_effect = control_causaloid
            .evaluate(&input_effect)
            .unwrap()
            .as_numerical()
            .unwrap();

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

/// The causal logic for the drug's effect.
/// This function checks the context to see if the drug was administered and returns the effect on blood pressure.
fn drug_effect_logic(
    _effect: &PropagatingEffect, // We don't need the incoming effect for this simple model
    context: &Arc<RwLock<BaseContext>>,
) -> Result<PropagatingEffect, CausalityError> {
    let mut drug_administered = false;

    let ctx = context.read().unwrap();

    // Search the context for the DRUG_ADMINISTERED_ID flag.
    for i in 0..ctx.number_of_nodes() {
        if let Some(node) = ctx.get_node(i)
            && let ContextoidType::Datoid(data_node) = node.vertex_type()
            && data_node.id() == DRUG_ADMINISTERED_ID
            && data_node.get_data() == 1.0
        {
            drug_administered = true;
            break;
        }
    }

    if drug_administered {
        // If the drug was given, it causes a 10-point drop in blood pressure.
        Ok(PropagatingEffect::Numerical(-10.0))
    } else {
        // If no drug was given, there is no effect.
        Ok(PropagatingEffect::Numerical(0.0))
    }
}

/// Creates a sample population of patients with different ages and blood pressures.
fn create_patient_population() -> Vec<BaseContext> {
    let mut population = Vec::new();
    let mut patient_id_counter = 1;

    // Tuples of (age, initial_bp)
    let patient_data = vec![
        (55.0, 145.0),
        (70.0, 150.0),
        (68.0, 155.0),
        (45.0, 130.0),
        (80.0, 160.0),
        (72.0, 148.0),
        (60.0, 140.0),
    ];

    for (age, bp) in patient_data {
        let mut context = BaseContext::with_capacity(patient_id_counter, "Patient", 5);
        patient_id_counter += 1;

        let age_datoid = Contextoid::new(
            patient_id_counter,
            ContextoidType::Datoid(Data::new(AGE_ID, age)),
        );
        context.add_node(age_datoid).unwrap();
        patient_id_counter += 1;

        let bp_datoid = Contextoid::new(
            patient_id_counter,
            ContextoidType::Datoid(Data::new(INITIAL_BP_ID, bp)),
        );
        context.add_node(bp_datoid).unwrap();
        patient_id_counter += 1;

        population.push(context);
    }

    population
}

/// Helper to extract the initial blood pressure from a patient's context.
fn get_patient_bp(context: &BaseContext) -> Option<f64> {
    for i in 0..context.number_of_nodes() {
        if let Some(node) = context.get_node(i)
            && let ContextoidType::Datoid(data_node) = node.vertex_type()
            && data_node.id() == INITIAL_BP_ID
        {
            return Some(data_node.get_data());
        }
    }
    None
}
