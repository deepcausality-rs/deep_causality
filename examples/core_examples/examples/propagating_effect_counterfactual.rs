/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{AlternatableValue, EffectLog, PropagatingEffect};
use deep_causality_haft::LogAddEntry;

fn main() {
    println!("--- Counterfactual Observation (Stateless) ---");

    // --------------------------------------------------------------------------------------------
    // ENGINEERING VALUE: Causal Reasoning ("What-If" Analysis)
    //
    // In complex systems, we often need to understand NOT just what happened, but what *would*
    // have happened if conditions were different.
    //
    // The `AlternatableValue` trait allows you to take an existing causal chain (process) and
    // computationally substitute a value (counterfactual value substitution), then observe the
    // downstream effects WITHOUT mutating source data or restarting the entire application.
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
    let mut log_a = EffectLog::new();
    log_a.add_entry(&format!("Initialized A with value: {}", a_factual));
    let effect_a: PropagatingEffect<i32> = PropagatingEffect::from_value_with_log(a_factual, log_a);

    // Chain computations with the fluent `bind`. The closure receives the wrapped value, the
    // threaded state, and the context; for a stateless effect the latter two are `()`.
    let effect_c_factual = effect_a.bind(|val_a, _state, _ctx| {
        let val_a = val_a.into_value().unwrap_or_default();
        // Step B: A * 2
        let val_b = val_a * 2;
        // Create effect and add log
        let mut log_b = EffectLog::new();
        log_b.add_entry(&format!("Computed B (A*2): {} * 2 = {}", val_a, val_b));
        let effect_b = PropagatingEffect::from_value_with_log(val_b, log_b);

        // Step C: B + 5
        effect_b.bind(|val_b, _state, _ctx| {
            let val_b = val_b.into_value().unwrap_or_default();
            let val_c = val_b + 5;
            let mut log_c = EffectLog::new();
            log_c.add_entry(&format!("Computed C (B+5): {} + 5 = {}", val_b, val_c));
            PropagatingEffect::from_value_with_log(val_c, log_c)
        })
    });

    println!(
        "  Factual Result: {:?}",
        effect_c_factual.value_cloned().unwrap()
    );
    println!("  Factual Logs:\n{}", effect_c_factual.logs());

    println!("\n2. Counterfactual World (Intervention):");
    println!("  Scenario: What if A had been 5?");

    // Start with the SAME initial setup
    let mut log_a = EffectLog::new();
    log_a.add_entry(&format!("Initialized A with value: {}", a_factual));
    let effect_a: PropagatingEffect<i32> = PropagatingEffect::from_value_with_log(a_factual, log_a); // Actually 10

    // Substitute the value immediately to change it from 10 to 5
    // The alternate_value method automatically adds a log entry "Intervened: value set to 5"
    let effect_a_intervened = effect_a.alternate_value(5);

    // Run the SAME logic
    let effect_c_counterfactual = effect_a_intervened.bind(|val_a, _state, _ctx| {
        let val_a = val_a.into_value().unwrap_or_default();
        // Step B: A * 2
        let val_b = val_a * 2;

        let mut log_b = EffectLog::new();
        log_b.add_entry(&format!("Computed B (A*2): {} * 2 = {}", val_a, val_b));
        let effect_b = PropagatingEffect::from_value_with_log(val_b, log_b);

        // Step C: B + 5
        effect_b.bind(|val_b, _state, _ctx| {
            let val_b = val_b.into_value().unwrap_or_default();
            let val_c = val_b + 5;
            let mut log_c = EffectLog::new();
            log_c.add_entry(&format!("Computed C (B+5): {} + 5 = {}", val_b, val_c));
            PropagatingEffect::from_value_with_log(val_c, log_c)
        })
    });

    println!(
        "  Counterfactual Result: {:?}",
        effect_c_counterfactual.value_cloned().unwrap()
    );

    // Inspect logs to see the full audit trail including initialization, intervention, and re-computation
    println!("\n3. Full Audit Trail (Counterfactual):");
    println!("{}", effect_c_counterfactual.logs());
}
