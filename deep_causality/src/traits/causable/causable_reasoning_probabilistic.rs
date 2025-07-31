/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AggregateLogic, Causable, CausalityError, NumericalValue, PropagatingEffect};

/// Evaluates a collection of `Causable` items against a specific `AggregateLogic`,
/// using a provided `threshold` to convert probabilistic effects into boolean values.
///
/// # Arguments
/// * `items` - A vector of references to `Causable` items.
/// * `effect` - The `PropagatingEffect` to pass to each item's `evaluate` method.
/// * `logic` - The aggregation logic to apply.
/// * `threshold` - The numerical threshold used to convert probabilistic effects to boolean.
///
/// # Returns
/// A `Result` containing the final `PropagatingEffect` outcome.
/// For `AggregateLogic::All`, it returns `PropagatingEffect::Probabilistic`.
/// For `Any`, `None`, and `Some(k)`, it returns `PropagatingEffect::Deterministic`.
/// Returns a `CausalityError` if any item returns an unsupported effect type.
pub(super) fn _evaluate_probabilistic_logic<T: Causable>(
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

    match logic {
        AggregateLogic::All => {
            // Preserve original behavior for All: multiply probabilities.
            let mut cumulative_prob: NumericalValue = 1.0;
            for cause in items {
                let evaluated_effect = cause.evaluate(effect)?;
                match evaluated_effect {
                    PropagatingEffect::Probabilistic(p) | PropagatingEffect::Numerical(p) => {
                        cumulative_prob *= p;
                    }
                    _ => {
                        return Err(CausalityError(format!(
                            "evaluate_probabilistic_propagation encountered a non-probabilistic effect: {evaluated_effect:?}. Only probabilistic or numerical effects are allowed for AggregateLogic::All."
                        )));
                    }
                }
            }
            Ok(PropagatingEffect::Probabilistic(cumulative_prob))
        }
        _ => {
            // For Any, None, Some(k): use threshold to convert to boolean and apply logic.
            let mut true_count = 0;
            for cause in items {
                let evaluated_effect = cause.evaluate(effect)?;
                let value = match evaluated_effect {
                    PropagatingEffect::Probabilistic(p) | PropagatingEffect::Numerical(p) => {
                        p > threshold
                    }
                    PropagatingEffect::Deterministic(b) => b,
                    _ => {
                        return Err(CausalityError(format!(
                            "evaluate_probabilistic_propagation encountered an unsupported effect: {evaluated_effect:?}. Only probabilistic, numerical, or deterministic effects are allowed for other AggregateLogic types."
                        )));
                    }
                };

                match logic {
                    AggregateLogic::Any => {
                        if value {
                            return Ok(PropagatingEffect::Deterministic(true));
                        }
                    }
                    AggregateLogic::None => {
                        if value {
                            return Ok(PropagatingEffect::Deterministic(false));
                        }
                    }
                    AggregateLogic::Some(k) => {
                        if value {
                            true_count += 1;
                            if true_count >= *k {
                                return Ok(PropagatingEffect::Deterministic(true));
                            }
                        }
                    }
                    _ => unreachable!(), // All is handled above.
                }
            }

            let final_result = match logic {
                AggregateLogic::Any => false,
                AggregateLogic::None => true,
                AggregateLogic::Some(k) => true_count >= *k,
                _ => unreachable!(), // All is handled above.
            };
            Ok(PropagatingEffect::Deterministic(final_result))
        }
    }
}
