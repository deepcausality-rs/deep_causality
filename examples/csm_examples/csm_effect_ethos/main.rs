/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::model::{get_alert_action, get_base_context, get_effect_ethos, get_test_causaloid};
use deep_causality::*;
use deep_causality_ethos::{DeonticInferable, TeloidModal};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

mod model;

fn main() {
    println!("--- Effect Ethos Example ---");
    println!();

    // Create a context and wrap it in an Arc for shared ownership
    let context_arc = Arc::new(RwLock::new(get_base_context()));

    // 1. Build Causaloid and CSM
    let causaloid = get_test_causaloid(Arc::clone(&context_arc));
    let default_data: PropagatingEffect<f64> = PropagatingEffect::pure(0.0);
    let state = CausalState::new(1, 1, default_data, causaloid, None);
    let action = get_alert_action();

    let csm = CSM::new(&[(&state, &action)]);

    // 2. Build an EffectEthos (deontic reasoning engine)
    let ethos = get_effect_ethos();

    // 3. Test data
    let test_data: PropagatingEffect<f64> = PropagatingEffect::pure(0.6);

    // 4. Demonstrate CSM evaluation (without ethos check)
    println!("=== Part 1: CSM Evaluation (no ethos check) ===");
    let csm_result = csm.eval_single_state(1, &test_data);
    match csm_result {
        Ok(_) => println!("CSM: Action triggered (temperature threshold exceeded)"),
        Err(e) => println!("CSM Error: {}", e),
    }
    println!();

    // 5. Demonstrate EffectEthos deontic reasoning (separate from CSM)
    println!("=== Part 2: EffectEthos Deontic Reasoning ===");

    // Create a proposed action that the ethos will evaluate
    let proposed_action = ProposedAction::new(1, "high_temp_alert".to_string(), HashMap::new());

    // Get the context for ethos evaluation
    let context = context_arc.read().unwrap();

    // Query the ethos for a verdict on the proposed action
    let verdict_result = ethos.evaluate_action(&proposed_action, &context, &["temperature"]);

    println!("Proposed action: {}", proposed_action.action_name());

    match verdict_result {
        Ok(verdict) => {
            println!("Ethos verdict outcome: {:?}", verdict.outcome());

            // Look up each norm in the justification to show human-readable reasons
            println!("Justification:");
            for norm_id in verdict.justification() {
                if let Some(norm) = ethos.get_norm(*norm_id) {
                    println!(
                        "  - Norm #{}: action '{}' is {:?}",
                        norm_id,
                        norm.action_identifier(),
                        norm.modality()
                    );
                } else {
                    println!("  - Norm #{} (details not found)", norm_id);
                }
            }

            match verdict.outcome() {
                TeloidModal::Impermissible => {
                    println!();
                    println!(">>> Action is FORBIDDEN by the ethos!");
                    println!("The ethos prevents triggering this alert based on deontic rules.");
                }
                TeloidModal::Obligatory => {
                    println!();
                    println!(">>> Action is REQUIRED by the ethos!");
                }
                TeloidModal::Optional(_) => {
                    println!();
                    println!(">>> Action is PERMITTED (optional) by the ethos.");
                }
            }
        }
        Err(e) => {
            println!("Ethos evaluation error: {:?}", e);
        }
    }

    println!();
    println!("--- Example Complete ---");
    println!();
    println!("Key insight: EffectEthos provides deontic reasoning (what SHOULD happen)");
    println!("separate from causal reasoning (what WILL happen given causes).");
}
