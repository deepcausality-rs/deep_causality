// SPDX-License-Identifier: MIT
// Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.

mod model;

use deep_causality::{CausalMonad, EffectValue};
use deep_causality_haft::MonadEffect3;

fn main() {
    println!("--- Structural Causal Model with Monadic Composition ---");
    println!("Demonstrating the 3 Rungs of Pearl's Ladder of Causation.\n");

    // --- Rung 1: Association (Seeing) ---
    // Question: "Given a person has high nicotine levels, what is their cancer risk?"
    // This is passive observation. We are seeing what the model associates with high nicotine.
    println!("--- Rung 1: Association (Seeing) ---");
    println!("Query: Given a person has high nicotine levels, what is their cancer risk?");

    // We assume we don't know the background genetic factor for this observation.
    let has_genetic_predisposition_rung1 = false;
    let nicotine_obs_rung1 = CausalMonad::pure(EffectValue::Numerical(0.8));

    let factual_effect = nicotine_obs_rung1
        .bind(model::smoking_logic)
        .bind(model::tar_logic)
        .bind(|has_tar| model::cancer_logic(has_tar, has_genetic_predisposition_rung1));

    println!(
        "Result: High cancer risk -> {}",
        factual_effect.value.as_bool().unwrap()
    );
    println!("Explanation:\n{}\n", factual_effect.explain());

    // --- Rung 2: Intervention (Doing) ---
    // Question: "What would the general cancer risk be if we forced everyone to stop smoking?"
    // This is a forward-looking, predictive question about an action.
    println!("--- Rung 2: Intervention (Doing) ---");
    println!("Query: What is the cancer risk if we force a person not to smoke?");

    let nicotine_obs_rung2 = CausalMonad::pure(EffectValue::Numerical(0.8));
    let has_genetic_predisposition_rung2 = false;

    let interventional_effect = nicotine_obs_rung2
        // We intervene on the output of the smoking logic.
        .bind(model::smoking_logic)
        // DO-OPERATOR: Force the value to `false`, regardless of the nicotine level.
        .intervene(EffectValue::Boolean(false)) // do(Smoking = false)
        .bind(model::tar_logic)
        .bind(|has_tar| model::cancer_logic(has_tar, has_genetic_predisposition_rung2));

    println!(
        "Result: Low cancer risk -> {}",
        !interventional_effect.value.as_bool().unwrap()
    );
    println!("Explanation:\n{}\n", interventional_effect.explain());

    // --- Rung 3: Counterfactual (Imagining) ---
    // Question: "We observed a patient has cancer. We know they were a smoker.
    // Would they have had cancer if they had not smoked?"
    // This is a retrospective question that requires using evidence from the real world.
    println!("--- Rung 3: Counterfactual (Imagining) ---");
    println!("Query: A smoker has cancer. Would they have had cancer if they hadn't smoked?");

    // Step 1: Abduction (Use the evidence to infer background factors)
    // We observed: Smoker=true, Cancer=true.
    // Our model says: Cancer = Tar OR Genetic.
    // Since Smoker=true -> Tar=true, this is consistent. But what if the cancer was
    // caused by genetics alone? We can infer the most likely background state.
    // For this example, let's assume we've inferred the patient has a genetic predisposition.
    let inferred_genetic_predisposition = true;
    println!(
        "1. Abduction: Inferred from observation that the patient has a genetic predisposition.\n"
    );

    // Step 2 & 3: Action & Prediction (Run the counterfactual world)
    println!("2. Action & Prediction: Simulating the counterfactual world...");
    let nicotine_obs_rung3 = CausalMonad::pure(EffectValue::Numerical(0.8));

    let counterfactual_effect = nicotine_obs_rung3
        .bind(model::smoking_logic)
        // INTERVENE on the past action.
        .intervene(EffectValue::Boolean(false)) // If they had not smoked...
        .bind(model::tar_logic)
        // ...but use the inferred background condition from the real world.
        .bind(|has_tar| model::cancer_logic(has_tar, inferred_genetic_predisposition));

    println!(
        "Result: They still would have had cancer -> {}",
        counterfactual_effect.value.as_bool().unwrap()
    );
    println!("Explanation:\n{}", counterfactual_effect.explain());
    println!("\nConclusion: The cancer was due to the genetic predisposition, not smoking.");
}
