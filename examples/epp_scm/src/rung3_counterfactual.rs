/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::model;
use deep_causality::*;
use std::sync::{Arc, RwLock};

pub fn run_rung3_counterfactual(explain: bool) {
    println!("--- Rung 3: Counterfactual ---");
    println!(
        "Query: Given a smoker with high tar, what would their cancer risk be if they hadn't smoked?"
    );

    // 1. Define the Causaloid with our contextual logic
    let cancer_risk_causaloid = Causaloid::new_with_context(
        1,
        model::contextual_cancer_risk_logic,
        Arc::new(RwLock::new(BaseContext::with_capacity(0, "temp", 1))), // Temporary context, will be replaced
        "Contextual Cancer Risk",
    );

    // 2. Create Factual Context: A person who smokes and has high tar.
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

    // 3. Create Counterfactual Context: Same person, but we hypothetically set smoking to zero.
    let mut counterfactual_context = factual_context.clone();
    // To update, we need to know the index. In this simple case, it's 0.
    // A real implementation might use a HashMap<ID, Index> for lookup.
    let new_nicotine_datoid = Contextoid::new(
        1,
        ContextoidType::Datoid(Data::new(model::NICOTINE_ID, 0.1)),
    );
    counterfactual_context
        .update_node(1, new_nicotine_datoid)
        .unwrap();

    // 4. Evaluate Both Scenarios
    let mut factual_causaloid = cancer_risk_causaloid.clone();
    factual_causaloid.set_context(Some(Arc::new(RwLock::new(factual_context))));

    let mut counterfactual_causaloid = cancer_risk_causaloid.clone();
    counterfactual_causaloid.set_context(Some(Arc::new(RwLock::new(counterfactual_context))));

    let factual_risk = factual_causaloid.evaluate(&PropagatingEffect::none());

    let counterfactual_risk = counterfactual_causaloid.evaluate(&PropagatingEffect::none());

    // 5. Assert and Explain
    println!(
        "Factual Result (smoker with high tar): Cancer risk is high -> {}",
        factual_risk.value.as_bool().unwrap()
    );
    if explain {
        // Explain the factual line of reasoning:
        println!("Explain:");
        println!("{}", factual_risk.explain());
    }
    println!(
        "Counterfactual Result (non-smoker with high tar): Cancer risk is high -> {}",
        counterfactual_risk.value.as_bool().unwrap()
    );

    assert_eq!(factual_risk.value, EffectValue::Boolean(true));
    assert_eq!(counterfactual_risk.value, EffectValue::Boolean(true));
    if explain {
        // Explain the counterfactual_risk line of reasoning:
        println!("Explain:");
        println!("{}", counterfactual_risk.explain());
    }

    println!(
        "Conclusion: The cancer risk remains high in the counterfactual case because the direct cause (tar) was not undone."
    );
    println!("\n");
}
