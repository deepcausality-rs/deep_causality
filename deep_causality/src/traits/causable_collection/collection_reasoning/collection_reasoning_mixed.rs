/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AggregateLogic, Causable, CausalityError, NumericalValue, PropagatingEffect};

pub(in crate::traits) fn _evaluate_mixed_logic<T: Causable>(
    items: Vec<&T>,
    effect: &PropagatingEffect,
    logic: &AggregateLogic,
    threshold: NumericalValue,
) -> Result<PropagatingEffect, CausalityError> {
    if items.is_empty() {
        return Err(CausalityError(
            "No Causaloids found to evaluate".to_string(),
        ));
    }

    let mut bool_results = Vec::new();
    let confidence_level = 0.95; // The desired confidence level.
    let epsilon = 0.05; // The indifference region.
    let max_samples = 1000; // The maximum number of samples to take.

    for cause in items {
        let evaluated_effect = cause.evaluate(effect)?;
        let bool_val = match evaluated_effect {
            PropagatingEffect::Deterministic(b) => b,
            PropagatingEffect::Probabilistic(p) => p > threshold,
            PropagatingEffect::UncertainFloat(u) => u.greater_than(threshold).to_bool(
                threshold,
                confidence_level,
                epsilon,
                max_samples,
            )?,
            PropagatingEffect::UncertainBool(u) => {
                u.to_bool(threshold, confidence_level, epsilon, max_samples)?
            }
            _ => return Err(CausalityError::new("Invalid effect type".to_string())),
        };
        bool_results.push(bool_val);
    }

    let final_result = match logic {
        AggregateLogic::All => bool_results.iter().all(|&v| v),
        AggregateLogic::Any => bool_results.iter().any(|&v| v),
        AggregateLogic::None => !bool_results.iter().any(|&v| v),
        AggregateLogic::Some(k) => bool_results.iter().filter(|&&v| v).count() >= *k,
    };

    Ok(PropagatingEffect::Deterministic(final_result))
}
