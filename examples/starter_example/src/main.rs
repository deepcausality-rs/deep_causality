/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_core::PropagatingEffect;

fn main() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        SCM EXAMPLE: Pearl's Ladder of Causation              ║");
    println!("║        Using only deep_causality_core                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Causal Model: Smoking (Nicotine) → Tar → Cancer");
    println!();

    rung1_association();
    rung2_intervention();
    rung3_counterfactual();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                      Summary                                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Rung 1 (Association): Observed correlation between smoking and cancer");
    println!("Rung 2 (Intervention): Demonstrated causal effect of stopping smoking");
    println!("Rung 3 (Counterfactual): Showed that alternative causes can dominate");
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
    println!("=== RUNG 1: ASSOCIATION (Seeing) ===");
    println!("Question: What is the cancer risk for observed smokers?");
    println!();

    // Observe a heavy smoker
    let heavy_smoker = PersonState {
        nicotine_level: 0.8,
        tar_level: 0.0, // Unknown before causal chain
        cancer_risk: 0.0,
    };

    // Observe a non-smoker
    let non_smoker = PersonState {
        nicotine_level: 0.1,
        tar_level: 0.0,
        cancer_risk: 0.0,
    };

    // Run causal chain for both
    let smoker_result = full_causal_chain(heavy_smoker);
    let non_smoker_result = full_causal_chain(non_smoker);

    let smoker_state = smoker_result.value.into_value().unwrap_or_default();
    let non_smoker_state = non_smoker_result.value.into_value().unwrap_or_default();

    println!("Heavy smoker (nicotine=0.8):");
    println!("  → Tar level: {:.2}", smoker_state.tar_level);
    println!("  → Cancer risk: {:.2}", smoker_state.cancer_risk);
    println!();
    println!("Non-smoker (nicotine=0.1):");
    println!("  → Tar level: {:.2}", non_smoker_state.tar_level);
    println!("  → Cancer risk: {:.2}", non_smoker_state.cancer_risk);
    println!();
    println!(
        "Conclusion: Smoking is ASSOCIATED with higher cancer risk ({:.0}% vs {:.0}%)",
        smoker_state.cancer_risk * 100.0,
        non_smoker_state.cancer_risk * 100.0
    );
    println!();
}

// ============================================================================
//  RUNG 2: INTERVENTION (Doing)
// ============================================================================
/// "What happens if I intervene and MAKE someone stop smoking?"
/// do(Smoking = 0)
///
/// This breaks the natural causal flow - we forcibly set a value.
fn rung2_intervention() {
    println!("=== RUNG 2: INTERVENTION (Doing) ===");
    println!("Question: What happens if we intervene and stop someone from smoking?");
    println!();

    // Original: heavy smoker
    let before_intervention = PersonState {
        nicotine_level: 0.8,
        tar_level: 0.0,
        cancer_risk: 0.0,
    };

    // INTERVENTION: do(Smoking = 0)
    // We forcibly set nicotine to 0, breaking natural dependencies
    let after_intervention = PersonState {
        nicotine_level: 0.0, // Intervention!
        tar_level: 0.0,
        cancer_risk: 0.0,
    };

    let before_result = full_causal_chain(before_intervention);
    let after_result = full_causal_chain(after_intervention);

    let before_state = before_result.value.into_value().unwrap_or_default();
    let after_state = after_result.value.into_value().unwrap_or_default();

    println!("Before intervention (heavy smoker):");
    println!("  → Cancer risk: {:.2}", before_state.cancer_risk);
    println!();
    println!("After intervention (forced cessation):");
    println!("  → Cancer risk: {:.2}", after_state.cancer_risk);
    println!();
    println!(
        "Conclusion: Stopping smoking CAUSES cancer risk reduction from {:.0}% to {:.0}%",
        before_state.cancer_risk * 100.0,
        after_state.cancer_risk * 100.0
    );
    println!();
}

// ============================================================================
//  RUNG 3: COUNTERFACTUAL (Imagining)
// ============================================================================
/// "Given that a patient has cancer and was a smoker, would they still have
///  gotten cancer if they had never smoked?"
///
/// This requires imagining an alternate reality where we change history.
fn rung3_counterfactual() {
    println!("=== RUNG 3: COUNTERFACTUAL (Imagining) ===");
    println!("Question: Would this cancer patient have gotten cancer if they never smoked?");
    println!();

    // FACTUAL WORLD: Patient was a heavy smoker AND had environmental tar exposure
    // (e.g., worked in asbestos factory)
    let factual_smoker_with_environmental = PersonState {
        nicotine_level: 0.8,
        tar_level: 0.7, // Pre-existing tar from environment
        cancer_risk: 0.0,
    };

    // COUNTERFACTUAL WORLD: Same patient, but imagine they never smoked
    // The environmental tar exposure remains!
    let counterfactual_non_smoker = PersonState {
        nicotine_level: 0.0, // Counterfactual: never smoked
        tar_level: 0.7,      // But environmental tar STILL exists
        cancer_risk: 0.0,
    };

    // For counterfactual, we evaluate tar->cancer directly (since tar already exists)
    let factual_result = tar_to_cancer(factual_smoker_with_environmental);
    let counterfactual_result = tar_to_cancer(counterfactual_non_smoker);

    let factual_state = factual_result.value.into_value().unwrap_or_default();
    let counterfactual_state = counterfactual_result.value.into_value().unwrap_or_default();

    println!("Factual world (smoker with environmental tar exposure):");
    println!("  → Nicotine: {:.1}", factual_state.nicotine_level);
    println!("  → Tar: {:.1}", factual_state.tar_level);
    println!("  → Cancer risk: {:.2}", factual_state.cancer_risk);
    println!();
    println!("Counterfactual world (never smoked, but same environmental exposure):");
    println!("  → Nicotine: {:.1}", counterfactual_state.nicotine_level);
    println!("  → Tar: {:.1}", counterfactual_state.tar_level);
    println!("  → Cancer risk: {:.2}", counterfactual_state.cancer_risk);
    println!();

    if (factual_state.cancer_risk - counterfactual_state.cancer_risk).abs() < 0.01 {
        println!("Conclusion: Cancer risk is THE SAME in both worlds!");
        println!("The direct cause (tar from environment) was NOT undone by stopping smoking.");
        println!("This patient would likely have gotten cancer regardless of smoking.");
    } else {
        println!("Conclusion: Cancer risk differs between worlds.");
        println!("Smoking contributed to the cancer risk.");
    }
    println!();
}

/// Our domain state representing observations about a person
#[derive(Debug, Clone, Copy, Default)]
struct PersonState {
    nicotine_level: f64, // 0.0 - 1.0 (observed smoking behavior)
    tar_level: f64,      // 0.0 - 1.0 (lung tar deposit)
    cancer_risk: f64,    // 0.0 - 1.0 (probability of cancer)
}

/// Compose the full causal chain: Smoking -> Tar -> Cancer
/// Uses monadic bind to chain effects
fn full_causal_chain(initial: PersonState) -> PropagatingEffect<PersonState> {
    let effect = PropagatingEffect::pure(initial);

    // Monadic composition: each step feeds into the next
    effect
        .bind(|state, _, _| smoking_to_tar(state.into_value().unwrap_or_default()))
        .bind(|state, _, _| tar_to_cancer(state.into_value().unwrap_or_default()))
}

/// Causal function: Nicotine → Tar accumulation
/// Higher nicotine exposure leads to tar buildup in lungs
fn smoking_to_tar(state: PersonState) -> PropagatingEffect<PersonState> {
    let tar_level = if state.nicotine_level > 0.5 {
        0.8 // High smoker -> high tar
    } else if state.nicotine_level > 0.2 {
        0.4 // Moderate smoker -> some tar
    } else {
        0.1 // Non-smoker or light -> minimal tar (environmental)
    };

    PropagatingEffect::pure(PersonState {
        nicotine_level: state.nicotine_level,
        tar_level,
        cancer_risk: state.cancer_risk,
    })
}

/// Causal function: Tar → Cancer Risk
/// Tar deposits in lungs directly cause cancer risk
fn tar_to_cancer(state: PersonState) -> PropagatingEffect<PersonState> {
    let cancer_risk = if state.tar_level > 0.6 {
        0.85 // High tar -> very high risk
    } else if state.tar_level > 0.3 {
        0.45 // Moderate tar -> elevated risk
    } else {
        0.15 // Low tar -> baseline risk
    };

    PropagatingEffect::pure(PersonState {
        nicotine_level: state.nicotine_level,
        tar_level: state.tar_level,
        cancer_risk,
    })
}
