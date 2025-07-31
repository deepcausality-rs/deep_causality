/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AggregateLogic, Causable, CausalityError, PropagatingEffect};

/// Evaluates a collection of `Causable` items against a specific `AggregateLogic`.
///
/// This is a private helper function that encapsulates the core reasoning logic,
/// allowing the public-facing trait method to remain a simple delegation.
/// It is optimized to short-circuit for performance where possible.
///
/// # Arguments
/// * `items` - A vector of references to `Causable` items.
/// * `effect` - The `PropagatingEffect` to pass to each item's `evaluate` method.
/// * `logic` - The aggregation logic to apply.
///
/// # Returns
/// A `Result` containing the final `PropagatingEffect::Deterministic` outcome
/// or a `CausalityError` if any item returns a non-deterministic effect.
pub fn _evaluate_deterministic_logic<T: Causable>(
    items: Vec<&T>,
    effect: &PropagatingEffect,
    logic: &AggregateLogic,
) -> Result<PropagatingEffect, CausalityError> {
    if items.is_empty() {
        return Err(CausalityError(
            "No Causaloids found to evaluate".to_string(),
        ));
    }

    let mut true_count = 0;

    for cause in items {
        let evaluated_effect = cause.evaluate(effect)?;

        let value = match evaluated_effect {
            PropagatingEffect::Deterministic(v) => v,
            _ => {
                // Strict contract: only deterministic effects are allowed.
                return Err(CausalityError(format!(
                    "evaluate_deterministic_propagation encountered a non-deterministic effect: {evaluated_effect:?}. Only Deterministic effects are allowed."
                )));
            }
        };

        match logic {
            AggregateLogic::All => {
                if !value {
                    // Short-circuit on the first false.
                    return Ok(PropagatingEffect::Deterministic(false));
                }
            }
            AggregateLogic::Any => {
                if value {
                    // Short-circuit on the first true.
                    return Ok(PropagatingEffect::Deterministic(true));
                }
            }
            AggregateLogic::None => {
                if value {
                    // Short-circuit on the first true, as this violates the None condition.
                    return Ok(PropagatingEffect::Deterministic(false));
                }
            }
            AggregateLogic::Some(k) => {
                if value {
                    true_count += 1;
                    if true_count >= *k {
                        // Short-circuit as soon as the threshold is met.
                        return Ok(PropagatingEffect::Deterministic(true));
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
        // If we got here for Some(k), it means the threshold was never met.
        AggregateLogic::Some(k) => true_count >= *k, // This will be false.
    };

    Ok(PropagatingEffect::Deterministic(final_result))
}
