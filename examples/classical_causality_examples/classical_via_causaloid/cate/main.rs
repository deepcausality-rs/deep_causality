/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use model::PatientContext;
use std::error::Error;
use std::sync::{Arc, RwLock};

mod model;

// Node indices of the contextoids a patient world carries: age, initial blood pressure, the BP
// change the drug produces when administered, and, in each trial arm, the treatment assignment.
const AGE: usize = 0;
const INITIAL_BP: usize = 1;
const DOSE: usize = 2;
const DRUG_ADMINISTERED: usize = 3;

// Define ID for the causaloid
const DRUG_EFFECT_CAUSALOID_ID: IdentificationValue = 10;

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n--- CATE Example: Effect of Medication on Blood Pressure for Patients > 65 ---");

    // 1. Define the population of patients
    let patient_population = model::create_patient_population()?;
    println!(
        "Created a population of {} patients.",
        patient_population.len()
    );

    // 2. Select the subgroup of interest (patients over 65)
    let ages = patient_population
        .iter()
        .map(|ctx| model::read(ctx, AGE).map(|age| (ctx, age)))
        .collect::<Result<Vec<_>, _>>()?;
    let subgroup: Vec<&PatientContext> = ages
        .into_iter()
        .filter(|&(_, age)| age > 65.0)
        .map(|(ctx, _)| ctx)
        .collect();
    println!(
        "Found {} patients in the subgroup (age > 65).",
        subgroup.len()
    );

    // 3. Run parallel counterfactuals for the subgroup
    let mut ites: Vec<f64> = Vec::new(); // To store Individual Treatment Effects

    for patient_context in subgroup {
        let initial_bp = model::read(patient_context, INITIAL_BP)?;

        // --- Create Counterfactual Contexts ---
        let treatment_context = model::arm(patient_context, true)?;
        let control_context = model::arm(patient_context, false)?;

        // --- Instantiate Causaloids for each scenario ---
        // New API: ContextualCausalFn = fn(CausalEffect<I>, S, Option<C>) -> PropagatingProcess<O, S, C>
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
        let y1_effect = model::value_of(&y1_res)?;

        let y0_res = control_causaloid.evaluate(&input_effect);
        let y0_effect = model::value_of(&y0_res)?;

        let y1 = initial_bp + y1_effect; // Potential outcome if treated
        let y0 = initial_bp + y0_effect; // Potential outcome if not treated

        // --- Calculate and Store ITE ---
        let ite = y1 - y0;
        ites.push(ite);
    }

    // 4. Aggregate and Conclude
    if !ites.is_empty() {
        let cate: f64 = deep_causality_stats::mean(&ites)?;
        println!("\n--- CATE Calculation Result ---");
        println!(
            "The Conditional Average Treatment Effect (CATE) for patients over 65 is: {:.2}",
            cate
        );
    } else {
        println!("\nNo patients found in the subgroup to calculate CATE.");
    }
    Ok(())
}
