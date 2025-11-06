/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{AggregateLogic, Causable, CausalityError, NumericalValue, PropagatingEffect};
use deep_causality_uncertain::Uncertain;

pub(in crate::traits) fn _evaluate_uncertain_logic<T: Causable>(
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

    let mut uncertain_bools: Vec<Uncertain<bool>> = Vec::new();

    for cause in items {
        let evaluated_effect = cause.evaluate(effect)?;
        match evaluated_effect {
            PropagatingEffect::UncertainBool(u) => {
                uncertain_bools.push(u);
            }
            PropagatingEffect::UncertainFloat(u) => {
                uncertain_bools.push(u.greater_than(threshold));
            }
            _ => {
                return Err(CausalityError::new(format!(
                    "Invalid effect type for uncertain evaluation: {:?}",
                    evaluated_effect
                )));
            }
        }
    }

    if uncertain_bools.is_empty() {
        // This case might be hit if all effects were of ignored types.
        // Returning an error might be better, but for now, let's stick to the original logic.
        return Err(CausalityError::new(
            "No uncertain-compatible effects found in the collection".to_string(),
        ));
    }

    let final_uncertain_bool = match logic {
        AggregateLogic::All => {
            let mut result = uncertain_bools.remove(0);
            for u in uncertain_bools {
                result = result & u;
            }
            result
        }
        AggregateLogic::Any => {
            let mut result = uncertain_bools.remove(0);
            for u in uncertain_bools {
                result = result | u;
            }
            result
        }
        AggregateLogic::None => {
            let mut any_result = uncertain_bools.remove(0);
            for u in uncertain_bools {
                any_result = any_result | u;
            }
            !any_result
        }
        AggregateLogic::Some(k) => {
            let confidence = 0.95; // Default confidence for hypothesis testing
            let epsilon = 0.05; // The indifference region.
            let max_samples = 1000; // The maximum number of samples to take.

            let bool_results: Result<Vec<bool>, _> = uncertain_bools
                .into_iter()
                .map(|u| u.to_bool(threshold, confidence, epsilon, max_samples))
                .collect();

            let bool_results = bool_results?;

            let true_count = bool_results.iter().filter(|&&b| b).count();
            Uncertain::<bool>::point(true_count >= *k)
        }
    };

    Ok(PropagatingEffect::UncertainBool(final_uncertain_bool))
}
