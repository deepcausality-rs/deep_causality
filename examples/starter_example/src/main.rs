/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # DeepCausality Starter Example: Pearl's Ladder of Causation
//!
//! This example demonstrates the three rungs of Pearl's Ladder using
//! `PropagatingEffect` and the `Intervenable` trait from `deep_causality_core`.
//!
//! **Causal Model:** Smoking (Nicotine) → Tar → Cancer
//!
//! Run with: `cargo run -p starter_example`

use deep_causality_core::{Intervenable, PropagatingEffect};

fn main() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        DeepCausality: Pearl's Ladder of Causation            ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Causal Model: Smoking (Nicotine) → Tar → Cancer");
    println!();

    rung1_association();
    rung2_intervention();
    rung3_counterfactual();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                         Summary                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Rung 1 (Association): Observed correlation between smoking and cancer");
    println!("Rung 2 (Intervention): Used intervene() to force tar removal mid-chain");
    println!("Rung 3 (Counterfactual): Same chain, intervene at start to simulate 'never smoked'");
    println!();
}

// ============================================================================
//  RUNG 1: ASSOCIATION (Seeing)
// ============================================================================
/// "What is the probability of cancer given that we observe smoking?"
/// P(Cancer | Smoking)
///
/// This is purely observational - we just watch and record.
fn rung1_association() {
    println!("═══ RUNG 1: ASSOCIATION (Seeing) ═══");
    println!("Question: What is the cancer risk for observed smokers vs non-smokers?");
    println!();

    // Run the causal chain for a heavy smoker
    let smoker_result = causal_chain(0.8); // High nicotine
    let non_smoker_result = causal_chain(0.1); // Low nicotine

    println!("Heavy smoker (nicotine=0.8):");
    println!("  → Cancer risk: {:.0}%", smoker_result * 100.0);
    println!();
    println!("Non-smoker (nicotine=0.1):");
    println!("  → Cancer risk: {:.0}%", non_smoker_result * 100.0);
    println!();
    println!(
        "Conclusion: Smoking is ASSOCIATED with higher cancer risk ({:.0}% vs {:.0}%)",
        smoker_result * 100.0,
        non_smoker_result * 100.0
    );
    println!();
}

// ============================================================================
//  RUNG 2: INTERVENTION (Doing)
// ============================================================================
/// "What happens if I intervene and MAKE someone stop smoking?"
/// do(Tar := 0.1)
///
/// This uses `intervene()` to force a value mid-chain, breaking natural flow.
fn rung2_intervention() {
    println!("═══ RUNG 2: INTERVENTION (Doing) ═══");
    println!("Question: What if we intervene and force tar removal mid-chain?");
    println!();

    // BEFORE: Natural causal chain for heavy smoker
    let before = PropagatingEffect::pure(0.8_f64) // Nicotine level
        .bind(|nic, _, _| {
            let n = nic.into_value().unwrap_or_default();
            PropagatingEffect::pure(nicotine_to_tar(n))
        })
        .bind(|tar, _, _| {
            let t = tar.into_value().unwrap_or_default();
            PropagatingEffect::pure(tar_to_cancer(t))
        });

    // AFTER: Same chain, but INTERVENE after nicotine→tar to force tar=0.1
    // This simulates: "What if we surgically removed the tar mid-process?"
    let after = PropagatingEffect::pure(0.8_f64) // Same nicotine level
        .bind(|nic, _, _| {
            let n = nic.into_value().unwrap_or_default();
            PropagatingEffect::pure(nicotine_to_tar(n)) // Tar would be 0.8
        })
        .intervene(0.1) // ← INTERVENTION: Force tar to 0.1 (medical treatment)
        .bind(|tar, _, _| {
            let t = tar.into_value().unwrap_or_default();
            PropagatingEffect::pure(tar_to_cancer(t))
        });

    let before_risk = before.value.into_value().unwrap_or_default();
    let after_risk = after.value.into_value().unwrap_or_default();

    println!("Before intervention (natural chain):");
    println!("  Nicotine(0.8) → Tar(0.8) → Cancer Risk: {:.0}%", before_risk * 100.0);
    println!();
    println!("After intervention:");
    println!("  Nicotine(0.8) → Tar(0.8) → [intervene(0.1)] → Cancer Risk: {:.0}%", after_risk * 100.0);
    println!();
    println!(
        "Conclusion: Intervention CAUSES cancer risk reduction from {:.0}% to {:.0}%",
        before_risk * 100.0,
        after_risk * 100.0
    );
    println!();
}

// ============================================================================
//  RUNG 3: COUNTERFACTUAL (Imagining)
// ============================================================================
/// "Given that a patient has cancer and was a smoker, would they still have
///  gotten cancer if they had never smoked?"
///
/// We use `intervene()` at the START of the chain to simulate "never smoked".
fn rung3_counterfactual() {
    println!("═══ RUNG 3: COUNTERFACTUAL (Imagining) ═══");
    println!("Question: Would this cancer patient have gotten cancer if they never smoked?");
    println!();

    // FACTUAL: Patient was a heavy smoker
    let factual = PropagatingEffect::pure(0.8_f64)
        .bind(|nic, _, _| {
            let n = nic.into_value().unwrap_or_default();
            PropagatingEffect::pure(nicotine_to_tar(n))
        })
        .bind(|tar, _, _| {
            let t = tar.into_value().unwrap_or_default();
            PropagatingEffect::pure(tar_to_cancer(t))
        });

    // COUNTERFACTUAL: Same chain, but intervene at the START
    // "What if nicotine had been 0 from the beginning?"
    let counterfactual = PropagatingEffect::pure(0.8_f64)
        .intervene(0.0) // ← Counterfactual: "Had they never smoked"
        .bind(|nic, _, _| {
            let n = nic.into_value().unwrap_or_default();
            PropagatingEffect::pure(nicotine_to_tar(n))
        })
        .bind(|tar, _, _| {
            let t = tar.into_value().unwrap_or_default();
            PropagatingEffect::pure(tar_to_cancer(t))
        });

    let factual_risk = factual.value.into_value().unwrap_or_default();
    let counterfactual_risk = counterfactual.value.into_value().unwrap_or_default();

    println!("Factual world (was a heavy smoker):");
    println!("  Nicotine(0.8) → Tar(0.8) → Cancer Risk: {:.0}%", factual_risk * 100.0);
    println!();
    println!("Counterfactual world (had they never smoked):");
    println!("  [intervene(0.0)] → Tar(0.1) → Cancer Risk: {:.0}%", counterfactual_risk * 100.0);
    println!();

    let causal_effect = factual_risk - counterfactual_risk;
    println!(
        "Individual Causal Effect (ICE): {:.0}% increased cancer risk from smoking",
        causal_effect * 100.0
    );
    println!();
}

// ============================================================================
//  CAUSAL FUNCTIONS
// ============================================================================

/// Full causal chain: Nicotine → Tar → Cancer Risk
fn causal_chain(nicotine: f64) -> f64 {
    let result = PropagatingEffect::pure(nicotine)
        .bind(|nic, _, _| {
            let n = nic.into_value().unwrap_or_default();
            PropagatingEffect::pure(nicotine_to_tar(n))
        })
        .bind(|tar, _, _| {
            let t = tar.into_value().unwrap_or_default();
            PropagatingEffect::pure(tar_to_cancer(t))
        });

    result.value.into_value().unwrap_or_default()
}

/// Causal mechanism: Nicotine → Tar accumulation
fn nicotine_to_tar(nicotine: f64) -> f64 {
    if nicotine > 0.5 {
        0.8 // Heavy smoker → high tar
    } else if nicotine > 0.2 {
        0.4 // Moderate → some tar
    } else {
        0.1 // Non-smoker → minimal (environmental baseline)
    }
}

/// Causal mechanism: Tar → Cancer Risk
fn tar_to_cancer(tar: f64) -> f64 {
    if tar > 0.6 {
        0.85 // High tar → very high risk
    } else if tar > 0.3 {
        0.45 // Moderate tar → elevated risk
    } else {
        0.15 // Low tar → baseline risk
    }
}
