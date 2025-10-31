/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use std::sync::{Arc, RwLock};

// Contextoid IDs
const NICOTINE_ID: IdentificationValue = 1;
const TAR_ID: IdentificationValue = 2;

/// A contextual causal function that determines cancer risk.
/// It prioritizes checking for tar, then for smoking.
fn contextual_cancer_risk_logic(
    _effect: &PropagatingEffect,
    context: &Arc<RwLock<BaseContext>>,
) -> Result<PropagatingEffect, CausalityError> {
    let mut tar_level = 0.0;
    let mut nicotine_level = 0.0;

    let ctx = context.read().unwrap();

    // Scan the context for relevant data.
    for i in 0..ctx.number_of_nodes() {
        if let Some(node) = ctx.get_node(i)
            && let ContextoidType::Datoid(data_node) = node.vertex_type()
        {
            match data_node.id() {
                TAR_ID => tar_level = data_node.get_data(),
                NICOTINE_ID => nicotine_level = data_node.get_data(),
                _ => (),
            }
        }
    }

    // Causal Logic: High tar is a direct cause of cancer risk, regardless of smoking.
    if tar_level > 0.6 {
        return Ok(PropagatingEffect::Deterministic(true));
    }
    // If tar is low, then smoking becomes the relevant factor.
    if nicotine_level > 0.6 {
        return Ok(PropagatingEffect::Deterministic(true));
    }

    Ok(PropagatingEffect::Deterministic(false))
}

pub fn run_rung3_counterfactual() {
    println!("--- Rung 3: Counterfactual ---");
    println!(
        "Query: Given a smoker with high tar, what would their cancer risk be if they hadn't smoked?"
    );

    // 1. Define the Causaloid with our contextual logic
    let cancer_risk_causaloid = Causaloid::new_with_context(
        1,
        contextual_cancer_risk_logic,
        Arc::new(RwLock::new(BaseContext::with_capacity(0, "temp", 1))), // Temporary context, will be replaced
        "Contextual Cancer Risk",
    );

    // 2. Create Factual Context: A person who smokes and has high tar.
    let mut factual_context = BaseContext::with_capacity(1, "Factual", 5);
    factual_context
        .add_node(Contextoid::new(
            1,
            ContextoidType::Datoid(Data::new(NICOTINE_ID, 0.8)),
        ))
        .unwrap();
    factual_context
        .add_node(Contextoid::new(
            2,
            ContextoidType::Datoid(Data::new(TAR_ID, 0.8)),
        ))
        .unwrap();

    // 3. Create Counterfactual Context: Same person, but we hypothetically set smoking to zero.
    let mut counterfactual_context = factual_context.clone();
    // To update, we need to know the index. In this simple case, it's 0.
    // A real implementation might use a HashMap<ID, Index> for lookup.
    let new_nicotine_datoid =
        Contextoid::new(1, ContextoidType::Datoid(Data::new(NICOTINE_ID, 0.1)));
    counterfactual_context
        .update_node(1, new_nicotine_datoid)
        .unwrap();

    // 4. Evaluate Both Scenarios
    let mut factual_causaloid = cancer_risk_causaloid.clone();
    factual_causaloid.set_context(Some(Arc::new(RwLock::new(factual_context))));

    let mut counterfactual_causaloid = cancer_risk_causaloid.clone();
    counterfactual_causaloid.set_context(Some(Arc::new(RwLock::new(counterfactual_context))));

    let factual_risk = factual_causaloid
        .evaluate(&PropagatingEffect::None)
        .unwrap();
    let counterfactual_risk = counterfactual_causaloid
        .evaluate(&PropagatingEffect::None)
        .unwrap();

    // 5. Assert and Explain
    println!(
        "Factual Result (smoker with high tar): Cancer risk is high -> {}",
        factual_risk.as_bool().unwrap()
    );
    println!(
        "Counterfactual Result (non-smoker with high tar): Cancer risk is high -> {}",
        counterfactual_risk.as_bool().unwrap()
    );

    assert_eq!(factual_risk, PropagatingEffect::Deterministic(true));
    assert_eq!(counterfactual_risk, PropagatingEffect::Deterministic(true));

    println!(
        "Conclusion: The cancer risk remains high in the counterfactual world because the direct cause (tar) was not undone."
    );
    println!("\n");
}
