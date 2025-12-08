/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{Intervenable, PropagatingEffectWitness};
use deep_causality_haft::{Applicative, LogAddEntry, Monad};

fn main() {
    println!("--- Counterfactual Observation (Stateless) ---");

    // --------------------------------------------------------------------------------------------
    // ENGINEERING VALUE: Causal Reasoning ("What-If" Analysis)
    //
    // In complex systems, we often need to understand NOT just what happened, but what *would*
    // have happened if conditions were different.
    //
    // The `Intervenable` trait allows you to take an existing causal chain (process) and
    // computationally "intervene" to override a value, then observe the downstream effects
    // WITHOUT mutating source data or restarting the entire application.
    //
    // This is fundamental for:
    // - **Explainable AI**: "Why did the drone crash? Because sensor A was 0. If it were 1..."
    // - **Testing**: Verifying edge cases that are hard to reproduce physically.
    // --------------------------------------------------------------------------------------------

    // 1. Define a Causal Model (A -> B -> C)
    // A: Initial Value
    // B: A * 2
    // C: B + 5

    println!("\n1. Factual World:");
    let a_factual = 10;

    // Start with pure value and Log initialization
    let mut effect_a = PropagatingEffectWitness::pure(a_factual);
    effect_a
        .logs
        .add_entry(&format!("Initialized A with value: {}", a_factual));

    // Chain computations
    let effect_c_factual = PropagatingEffectWitness::bind(effect_a, |val_a| {
        // Step B: A * 2
        let val_b = val_a * 2;
        // Create effect and add log
        let mut effect_b = PropagatingEffectWitness::pure(val_b);
        effect_b
            .logs
            .add_entry(&format!("Computed B (A*2): {} * 2 = {}", val_a, val_b));

        // Step C: B + 5
        PropagatingEffectWitness::bind(effect_b, |val_b| {
            let val_c = val_b + 5;
            let mut effect_c = PropagatingEffectWitness::pure(val_c);
            effect_c
                .logs
                .add_entry(&format!("Computed C (B+5): {} + 5 = {}", val_b, val_c));
            effect_c
        })
    });

    println!(
        "  Factual Result: {:?}",
        effect_c_factual.value.into_value().unwrap()
    );
    println!("  Factual Logs:\n{}", effect_c_factual.logs);

    println!("\n2. Counterfactual World (Intervention):");
    println!("  Scenario: What if A had been 5?");

    // Start with the SAME initial setup
    let mut effect_a = PropagatingEffectWitness::pure(a_factual); // Actually 10
    effect_a
        .logs
        .add_entry(&format!("Initialized A with value: {}", a_factual));

    // INTERVENE immediately to change the value from 10 to 5
    // The intervene method automatically adds a log entry "Intervened: value set to 5"
    let effect_a_intervened = effect_a.intervene(5);

    // Run the SAME logic
    let effect_c_counterfactual = PropagatingEffectWitness::bind(effect_a_intervened, |val_a| {
        // Step B: A * 2
        let val_b = val_a * 2;

        let mut effect_b = PropagatingEffectWitness::pure(val_b);
        effect_b
            .logs
            .add_entry(&format!("Computed B (A*2): {} * 2 = {}", val_a, val_b));

        // Step C: B + 5
        PropagatingEffectWitness::bind(effect_b, |val_b| {
            let val_c = val_b + 5;
            let mut effect_c = PropagatingEffectWitness::pure(val_c);
            effect_c
                .logs
                .add_entry(&format!("Computed C (B+5): {} + 5 = {}", val_b, val_c));
            effect_c
        })
    });

    println!(
        "  Counterfactual Result: {:?}",
        effect_c_counterfactual.value.into_value().unwrap()
    );

    // Inspect logs to see the full audit trail including initialization, intervention, and re-computation
    println!("\n3. Full Audit Trail (Counterfactual):");
    println!("{}", effect_c_counterfactual.logs);
}
