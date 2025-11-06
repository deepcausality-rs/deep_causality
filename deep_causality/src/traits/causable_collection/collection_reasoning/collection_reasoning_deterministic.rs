/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AggregateLogic, CausalMonad, CausalityError, MonadicCausable, PropagatingEffect};

pub(in crate::traits) fn _evaluate_deterministic_logic<T: MonadicCausable<CausalMonad>>(
    items: Vec<&T>,
    effect: &PropagatingEffect,
    logic: &AggregateLogic,
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

    let mut true_count = 0;

    for cause in items {
        let evaluated_effect = cause.evaluate_monadic(effect.clone());

        if let Some(err) = evaluated_effect.error {
            return PropagatingEffect {
                value: crate::EffectValue::None,
                error: Some(err),
                logs: evaluated_effect.logs,
            };
        }

        let value = match evaluated_effect.value {
            crate::EffectValue::Deterministic(v) => v,
            _ => {
                // Strict contract: only deterministic effects are allowed.
                return PropagatingEffect {
                    value: crate::EffectValue::None,
                    error: Some(CausalityError(format!(
                        "evaluate_deterministic_propagation encountered a non-deterministic effect: {:?}. Only Deterministic effects are allowed.", evaluated_effect.value
                    ))),
                    logs: evaluated_effect.logs,
                };
            }
        };

        match logic {
            AggregateLogic::All => {
                if !value {
                    // Short-circuit on the first false.
                    return PropagatingEffect::deterministic(false, evaluated_effect.logs);
                }
            }
            AggregateLogic::Any => {
                if value {
                    // Short-circuit on the first true.
                    return PropagatingEffect::deterministic(true, evaluated_effect.logs);
                }
            }
            AggregateLogic::None => {
                if value {
                    // Short-circuit on the first true, as this violates the None condition.
                    return PropagatingEffect::deterministic(false, evaluated_effect.logs);
                }
            }
            AggregateLogic::Some(k) => {
                if value {
                    true_count += 1;
                    if true_count >= *k {
                        // Short-circuit as soon as the threshold is met.
                        return PropagatingEffect::deterministic(true, evaluated_effect.logs);
                    }
                }
            }
        }
    }

    // If the loop completes, determine the final result for non-short-circuited paths.
    let final_result = match logic {
        // If we got here for All, it means no false values were found.
        AggregateLogic::All => true,
        // If we got here for Any, it means no true values were found.
        AggregateLogic::Any => false,
        // If we got here for None, it means no true values were found.
        AggregateLogic::None => true,
        // If we got here for Some(k) and the loop completed, it means the threshold was never met.
        AggregateLogic::Some(k) => true_count >= *k, // This will be false if not met.
    };

    PropagatingEffect::deterministic(final_result, effect.logs.clone())
}
