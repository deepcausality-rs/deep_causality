/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::model;
use deep_causality::*;
use std::sync::{Arc, RwLock};

pub fn run_rung3_counterfactual(_explain: bool) {
    println!("--- Rung 3: Counterfactual ---");
    println!(
        "Query: Given a smoker with high tar, what would their cancer risk be if they hadn't smoked?"
    );

    // 1. Create Factual Context: A person who smokes and has high tar.
    let mut factual_context = BaseContext::with_capacity(1, "Factual", 5);
    factual_context
        .add_node(Contextoid::new(
            1,
            ContextoidType::Datoid(Data::new(model::NICOTINE_ID, 0.8)),
        ))
        .unwrap();
    factual_context
        .add_node(Contextoid::new(
            2,
            ContextoidType::Datoid(Data::new(model::TAR_ID, 0.8)),
        ))
        .unwrap();

    // 2. Create Counterfactual Context: Same person, but we hypothetically set smoking to zero.
    let mut counterfactual_context = factual_context.clone();
    let new_nicotine_datoid = Contextoid::new(
        1,
        ContextoidType::Datoid(Data::new(model::NICOTINE_ID, 0.1)),
    );
    counterfactual_context
        .update_node(1, new_nicotine_datoid)
        .unwrap();

    // 3. Create causaloids for each context
    let factual_causaloid =
        model::get_contextual_cancer_causaloid(Arc::new(RwLock::new(factual_context)));
    let counterfactual_causaloid =
        model::get_contextual_cancer_causaloid(Arc::new(RwLock::new(counterfactual_context)));

    // 4. Evaluate Both Scenarios
    let input_effect: PropagatingEffect<f64> = PropagatingEffect::pure(0.0);

    let factual_result = factual_causaloid.evaluate(&input_effect);
    let counterfactual_result = counterfactual_causaloid.evaluate(&input_effect);

    let factual_risk = factual_result.value.into_value().unwrap_or(false);
    let counterfactual_risk = counterfactual_result.value.into_value().unwrap_or(false);

    // 5. Assert and Explain
    println!(
        "Factual Result (smoker with high tar): Cancer risk is high -> {}",
        factual_risk
    );
    println!(
        "Counterfactual Result (non-smoker with high tar): Cancer risk is high -> {}",
        counterfactual_risk
    );

    assert!(factual_risk, "Expected high cancer risk in factual case");
    assert!(
        counterfactual_risk,
        "Expected high cancer risk in counterfactual case (tar still present)"
    );

    println!(
        "Conclusion: The cancer risk remains high in the counterfactual case because the direct cause (tar) was not undone."
    );
    println!("\n");
}
