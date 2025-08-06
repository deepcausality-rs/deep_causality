/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AggregateLogic, Causable, CausalityError, NumericalValue, PropagatingEffect};

/// Evaluates a collection of `Causable` items that may contain a mix of deterministic and
/// probabilistic effects, aggregating them into a final deterministic outcome.
///
/// This is a private helper function that encapsulates the core reasoning logic,
/// allowing the public-facing trait method to remain a simple delegation.
/// It is optimized to short-circuit for performance where possible.
///
/// # Arguments
/// * `items` - A vector of references to `Causable` items.
/// * `effect` - The `PropagatingEffect` to pass to each item's `evaluate` method.
/// * `logic` - The aggregation logic to apply.
/// * `threshold` - The numerical threshold used to convert probabilistic effects to boolean.
///
/// # Returns
/// A `Result` containing the final `PropagatingEffect` outcome.
/// For `AggregateLogic::All`, it returns `PropagatingEffect::Deterministic` after applying the threshold.
/// For `Any`, `None`, and `Some(k)`, it returns `PropagatingEffect::Deterministic`.
/// Returns a `CausalityError` if any item returns an unsupported effect type.
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

    match logic {
        AggregateLogic::All => {
            let mut cumulative_prob: NumericalValue = 1.0;
            for cause in items {
                let current_effect = cause.evaluate(effect)?;
                let current_prob = match current_effect {
                    PropagatingEffect::Deterministic(true) => 1.0,
                    PropagatingEffect::Deterministic(false) => 0.0,
                    PropagatingEffect::Probabilistic(p) => p,
                    PropagatingEffect::Numerical(p) => p,
                    _ => {
                        return Err(CausalityError(format!(
                            "evaluate_mixed_propagation encountered an unsupported effect: {current_effect:?}. Only probabilistic, deterministic, or numerical effects are allowed."
                        )));
                    }
                };
                cumulative_prob *= current_prob;
            }
            Ok(PropagatingEffect::Deterministic(
                cumulative_prob > threshold,
            ))
        }
        _ => {
            let mut true_count = 0;
            for cause in items {
                let evaluated_effect = cause.evaluate(effect)?;
                let value = match evaluated_effect {
                    PropagatingEffect::Deterministic(b) => b,
                    PropagatingEffect::Probabilistic(p) | PropagatingEffect::Numerical(p) => {
                        p > threshold
                    }
                    _ => {
                        return Err(CausalityError(format!(
                            "evaluate_mixed_propagation encountered an unsupported effect: {evaluated_effect:?}. Only probabilistic, numerical, or deterministic effects are allowed."
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
