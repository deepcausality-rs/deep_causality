/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{MaybeUncertain, Uncertain, UncertainError};

/// Aspirin Headache Trial Analysis
///
/// This example demonstrates how to use `MaybeUncertain<f64>` to model pain reduction
/// data in a clinical trial where data presence itself can be uncertain.
/// It showcases various constructors, arithmetic operations, and the `lift_to_uncertain`
/// method to assess drug effectiveness under real-world data conditions.
fn main() -> Result<(), UncertainError> {
    println!("Aspirin Headache Trial Analysis");
    println!("=====================================\n");

    // --- 1. Modeling Patient Data with MaybeUncertain<f64> ---
    println!("--- Patient Pain Reduction Data ---");

    // Patient A: Control Group (Placebo Effect), certainly present, certain value
    let patient_a_pain_reduction = MaybeUncertain::<f64>::from_value(0.5);
    println!(
        "Patient A (Control): {:?}
",
        patient_a_pain_reduction.sample()?
    );

    // Patient B: Aspirin Group (Strong Responder), certainly present, uncertain value
    let patient_b_pain_reduction =
        MaybeUncertain::<f64>::from_uncertain(Uncertain::normal(6.0, 2.0));
    println!(
        "Patient B (Aspirin): {:?}
",
        patient_b_pain_reduction.sample()?
    );

    // Patient C: Aspirin Group (Weak Responder / Intermittent Reporting)
    // Low probability of reporting reduction, and if reported, it's small.
    let patient_c_pain_reduction =
        MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.3, Uncertain::normal(1.0, 0.5));
    println!(
        "Patient C (Aspirin): {:?}
",
        patient_c_pain_reduction.sample()?
    );

    // Patient D: Aspirin Group (Moderate Responder / Good Reporting)
    // High probability of reporting reduction, moderate effect.
    let patient_d_pain_reduction =
        MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.8, Uncertain::normal(4.0, 2.5));
    println!(
        "Patient D (Aspirin): {:?}
",
        patient_d_pain_reduction.sample()?
    );

    // Patient E: Control Group (Missing Data), certainly absent
    let patient_e_pain_reduction = MaybeUncertain::<f64>::always_none();
    println!(
        "Patient E (Control): {:?}
",
        patient_e_pain_reduction.sample()?
    );

    // --- 2. Assessing Data Presence Probabilities ---
    println!("--- Data Presence Probabilities ---");
    println!(
        "Patient A data is_some: {:.1}%\n",
        patient_a_pain_reduction
            .is_some()
            .estimate_probability(1000)?
            * 100.0
    );
    println!(
        "Patient C data is_some: {:.1}%\n",
        patient_c_pain_reduction
            .is_some()
            .estimate_probability(1000)?
            * 100.0
    );
    println!(
        "Patient E data is_none: {:.1}%\n",
        patient_e_pain_reduction
            .is_none()
            .estimate_probability(1000)?
            * 100.0
    );

    // --- 3. Arithmetic Operations with None Propagation ---
    println!("--- Arithmetic Operations (None Propagation) ---");

    // Summing pain reductions - if any is None, the result is None
    let sum_ab = patient_a_pain_reduction.clone() + patient_b_pain_reduction.clone();
    println!(
        "Sum A + B: {:?}
",
        sum_ab.sample()?
    );

    let sum_ac = patient_a_pain_reduction.clone() + patient_c_pain_reduction.clone();
    println!(
        "Sum A + C: {:?}
",
        sum_ac.sample()?
    ); // Likely None due to C's low presence probability

    let neg_b = -patient_b_pain_reduction.clone();
    println!(
        "Neg B: {:?}
",
        neg_b.sample()?
    );

    // --- 4. lift_to_uncertain() for Data Reliability ---
    println!("--- lift_to_uncertain() for Data Reliability ---");

    // Attempt to lift Patient D's data with a reasonable threshold (should succeed)
    match patient_d_pain_reduction.lift_to_uncertain(0.7, 0.95, 0.05, 1000) {
        Ok(uncertain_reduction) => {
            println!(
                "Patient D: Successfully lifted data. Mean reduction: {:.2}
",
                uncertain_reduction.expected_value(1000)?
            );
        }
        Err(e) => println!(
            "Patient D: Failed to lift data: {}
",
            e
        ),
    }

    // Attempt to lift Patient C's data with a high threshold (should fail)
    match patient_c_pain_reduction.lift_to_uncertain(0.8, 0.95, 0.05, 1000) {
        Ok(uncertain_reduction) => {
            println!(
                "Patient C: Successfully lifted data. Mean reduction: {:.2}
",
                uncertain_reduction.expected_value(1000)?
            );
        }
        Err(e) => println!(
            "Patient C: Failed to lift data: {}
",
            e
        ),
    }

    // Attempt to lift Patient E's data (always fails)
    match patient_e_pain_reduction.lift_to_uncertain(0.1, 0.95, 0.05, 1000) {
        Ok(uncertain_reduction) => {
            println!(
                "Patient E: Successfully lifted data. Mean reduction: {:.2}
",
                uncertain_reduction.expected_value(1000)?
            );
        }
        Err(e) => println!(
            "Patient E: Failed to lift data: {}
",
            e
        ),
    }

    // --- 5. Demonstrating Drug Effectiveness ---
    println!("\n--- Demonstrating Aspirin Effectiveness ---");

    // Collect and lift data for Aspirin group (only reliable data)
    let mut aspirin_reductions: Vec<Uncertain<f64>> = Vec::new();

    if let Ok(reduction) = patient_b_pain_reduction.lift_to_uncertain(0.9, 0.95, 0.05, 1000) {
        aspirin_reductions.push(reduction);
    }
    if let Ok(reduction) = patient_d_pain_reduction.lift_to_uncertain(0.7, 0.95, 0.05, 1000) {
        aspirin_reductions.push(reduction);
    }

    let control_reduction = patient_a_pain_reduction
        .clone()
        .lift_to_uncertain(0.9, 0.95, 0.05, 1000)?;

    if aspirin_reductions.is_empty() {
        println!("Not enough reliable Aspirin data to draw conclusions.");
    } else {
        let num_aspirin_reductions = aspirin_reductions.len() as f64;
        let total_aspirin_reduction = aspirin_reductions
            .into_iter()
            .reduce(|acc, r| acc + r)
            .unwrap();
        let avg_aspirin_reduction =
            total_aspirin_reduction / Uncertain::<f64>::point(num_aspirin_reductions);

        println!(
            "\nAverage Aspirin Group Pain Reduction: {:.2}
",
            avg_aspirin_reduction.expected_value(1000)?
        );
        println!(
            "Average Control Group Pain Reduction: {:.2}
",
            control_reduction.expected_value(1000)?
        );

        // Compare Aspirin vs Control
        let aspirin_better_than_control =
            avg_aspirin_reduction.greater_than(control_reduction.expected_value(1000)?);

        let confidence_aspirin_better =
            aspirin_better_than_control.estimate_probability(1000)? * 100.0;

        println!(
            "Confidence that Aspirin is better than Control: {:.1}%\n",
            confidence_aspirin_better
        );

        if aspirin_better_than_control.probability_exceeds(0.9, 0.95, 0.05, 1000)? {
            println!("✅ Conclusion: Aspirin reduces headache pain within uncertainty bounds!");
        } else {
            println!(
                "❌ Conclusion: Evidence is insufficient to confidently say Aspirin reduces headache pain."
            );
        }
    }

    Ok(())
}
