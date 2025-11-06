/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AggregateLogic, CausalMonad, CausalityError, MonadicCausable, NumericalValue, PropagatingEffect};

pub(in crate::traits) fn _evaluate_mixed_logic<T: MonadicCausable<CausalMonad>>(
    items: Vec<&T>,
    effect: &PropagatingEffect,
    logic: &AggregateLogic,
    threshold: NumericalValue,
) -> PropagatingEffect {
    if items.is_empty() {
        return PropagatingEffect {
            value: crate::EffectValue::None,
            error: Some(CausalityError(
                "No Causaloids found to evaluate".to_string(),
            )),
            logs: effect.logs.clone(),
        };
    }

    let mut bool_results = Vec::new();
    let confidence_level = 0.95; // The desired confidence level.
    let epsilon = 0.05; // The indifference region.
    let max_samples = 1000; // The maximum number of samples to take.

    for cause in items {
        let evaluated_effect = cause.evaluate_monadic(effect.clone());

        if let Some(err) = evaluated_effect.error {
            return PropagatingEffect {
                value: crate::EffectValue::None,
                error: Some(err),
                logs: evaluated_effect.logs,
            };
        }

        let bool_val = match evaluated_effect.value {
            crate::EffectValue::Deterministic(b) => b,
            crate::EffectValue::Probabilistic(p) => p > threshold,
            crate::EffectValue::UncertainFloat(u) => {
                match u.greater_than(threshold).to_bool(
                    threshold,
                    confidence_level,
                    epsilon,
                    max_samples,
                ) {
                    Ok(b) => b,
                    Err(e) => return PropagatingEffect {
                        value: crate::EffectValue::None,
                        error: Some(e),
                        logs: evaluated_effect.logs,
                    },
                }
            }
            crate::EffectValue::UncertainBool(u) => {
                match u.to_bool(threshold, confidence_level, epsilon, max_samples) {
                    Ok(b) => b,
                    Err(e) => return PropagatingEffect {
                        value: crate::EffectValue::None,
                        error: Some(e),
                        logs: evaluated_effect.logs,
                    },
                }
            }
            _ => return PropagatingEffect {
                value: crate::EffectValue::None,
                error: Some(CausalityError::new("Invalid effect type".to_string())),
                logs: evaluated_effect.logs,
            },
        };
        bool_results.push(bool_val);
    }

    let final_result = match logic {
        AggregateLogic::All => bool_results.iter().all(|&v| v),
        AggregateLogic::Any => bool_results.iter().any(|&v| v),
        AggregateLogic::None => !bool_results.iter().any(|&v| v),
        AggregateLogic::Some(k) => bool_results.iter().filter(|&&v| v).count() >= *k,
    };

    PropagatingEffect::deterministic(final_result, effect.logs.clone())
}
