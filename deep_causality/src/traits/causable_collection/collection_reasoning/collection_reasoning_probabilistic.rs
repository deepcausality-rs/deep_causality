/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AggregateLogic, Causable, CausalityError, NumericalValue, PropagatingEffect};

pub(in crate::traits) fn _evaluate_probabilistic_logic<T: Causable>(
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

    let mut probabilities = Vec::new();
    let num_samples = 100; // Number of samples for estimation

    for cause in items {
        let evaluated_effect = cause.evaluate(effect)?;
        let prob = match evaluated_effect {
            PropagatingEffect::Deterministic(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            }
            PropagatingEffect::Probabilistic(p) => p,
            PropagatingEffect::UncertainFloat(u) => {
                u.estimate_probability_exceeds(threshold, num_samples)?
            }
            PropagatingEffect::UncertainBool(u) => u.estimate_probability(num_samples)?,
            _ => return Err(CausalityError::new("Invalid effect type".to_string())),
        };
        probabilities.push(prob);
    }

    let final_prob = match logic {
        AggregateLogic::All => probabilities.iter().product(),
        AggregateLogic::Any => 1.0 - probabilities.iter().map(|p| 1.0 - p).product::<f64>(),
        AggregateLogic::None => {
            1.0 - (1.0 - probabilities.iter().map(|p| 1.0 - p).product::<f64>())
        }
        AggregateLogic::Some(k) => {
            // This is a simplification. A full implementation would use binomial distribution.
            let count = probabilities.iter().filter(|&&p| p > 0.5).count();
            if count >= *k { 1.0 } else { 0.0 }
        }
    };

    Ok(PropagatingEffect::Probabilistic(final_prob))
}
