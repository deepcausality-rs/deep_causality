/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Causable, CausalityError, IdentificationValue, NumericalValue, PropagatingEffect};

/// Provides default implementations for reasoning over collections of `Causable` items.
///
/// Any collection type that implements the basic accessor methods (`len`, `is_empty`,
/// `to_vec`, `get_all_items`) will automatically gain a suite of useful default
/// methods for inspecting the collective state of its `Causable` elements.
pub trait CausableReasoning<T>
where
    T: Causable,
{
    //
    // These methods must be implemented by the collection type.
    // See deep_causality/src/extensions/causable/mod.rs
    //

    /// Returns the total number of `Causable` items in the collection.
    fn len(&self) -> usize;

    /// Checks if the collection of `Causable` items is empty.
    fn is_empty(&self) -> bool;

    /// Creates a new vector containing the `Causable` items from the collection.
    fn to_vec(&self) -> Vec<T>;

    /// Returns a vector of references to all `Causable` items in the collection.
    /// This is the primary accessor used by the trait's default methods.
    fn get_all_items(&self) -> Vec<&T>;

    /// Returns a reference to a `Causable` item by its ID, if found.
    fn get_item_by_id(&self, id: IdentificationValue) -> Option<&T>;

    //
    // Default implementations for all other methods are provided below.
    //

    /// Evaluates a linear chain of causes where each link is strictly expected to be deterministic.
    ///
    /// The chain is considered active only if every single cause in the collection
    /// evaluates to `PropagatingEffect::Deterministic(true)`. If any cause evaluates
    /// to `Deterministic(false)`, the chain evaluation stops and returns that effect.
    ///
    /// # Arguments
    /// * `effect` - A single `PropagatingEffect` object (e.g., a Map or Graph) that all causes will use.
    ///
    /// # Errors
    /// Returns a `CausalityError` if any cause in the chain produces a non-deterministic effect.
    fn evaluate_deterministic_propagation(
        &self,
        effect: &PropagatingEffect,
    ) -> Result<PropagatingEffect, CausalityError> {
        for cause in self.get_all_items() {
            let effect = cause.evaluate(effect)?;

            // This function enforces a strict deterministic contract.
            let resolved_effect = match effect {
                PropagatingEffect::RelayTo(target_id, inner_effect) => {
                    let target_causaloid =
                        self.get_item_by_id(target_id as u64).ok_or_else(|| {
                            CausalityError(format!(
                                "Relay target causaloid with ID {target_id} not found."
                            ))
                        })?;
                    target_causaloid.evaluate(&inner_effect)?
                }
                _ => effect,
            };

            // This function enforces a strict deterministic contract.
            match resolved_effect {
                PropagatingEffect::Deterministic(true) => {
                    // The link is active, continue to the next one.
                    continue;
                }
                PropagatingEffect::Deterministic(false) => {
                    // The chain is deterministically broken. This is a valid final outcome.
                    return Ok(PropagatingEffect::Deterministic(false));
                }
                _ => {
                    // Any other effect type is a contract violation for this function.
                    return Err(CausalityError(format!(
                        "evaluate_deterministic_propagation encountered a non-deterministic effect: {resolved_effect:?}. Only Deterministic effects are allowed."
                    )));
                }
            }
        }

        // If the entire loop completes, all links were deterministically true.
        Ok(PropagatingEffect::Deterministic(true))
    }

    /// Evaluates a linear chain of causes where each link is expected to be probabilistic.
    ///
    /// This method aggregates the effects by multiplying their probabilities, assuming
    /// independence. It handles deterministic effects by treating `true` as a probability
    /// of 1.0 and `false` as 0.0.
    ///
    /// # Arguments
    /// * `effect` - A single `PropagatingEffect` object that all causes will use.
    ///
    /// # Errors
    /// Returns a `CausalityError` if a `ContextualLink` is encountered.
    fn evaluate_probabilistic_propagation(
        &self,
        effect: &PropagatingEffect,
    ) -> Result<PropagatingEffect, CausalityError> {
        let mut cumulative_prob: NumericalValue = 1.0;

        for cause in self.get_all_items() {
            let effect = cause.evaluate(effect)?;

            let resolved_effect = match effect {
                PropagatingEffect::RelayTo(target_id, inner_effect) => {
                    let target_causaloid = self
                        .get_item_by_id(target_id as IdentificationValue)
                        .ok_or_else(|| {
                            CausalityError(format!(
                                "Relay target causaloid with ID {target_id} not found."
                            ))
                        })?;
                    target_causaloid.evaluate(&inner_effect)?
                }
                _ => effect,
            };

            match resolved_effect {
                PropagatingEffect::Probabilistic(p) => {
                    cumulative_prob *= p;
                }
                PropagatingEffect::Deterministic(true) => {
                    // This is equivalent to multiplying by 1.0, so we do nothing and continue.
                }
                PropagatingEffect::Deterministic(false) => {
                    // If any link is deterministically false, the entire chain's probability is zero.
                    return Ok(PropagatingEffect::Probabilistic(0.0));
                }
                PropagatingEffect::Halting => {
                    // Halting always takes precedence and stops the chain.
                    return Ok(PropagatingEffect::Halting);
                }
                PropagatingEffect::ContextualLink(_, _) => {
                    // Contextual links are not meaningful in a probabilistic aggregation.
                    return Err(CausalityError(
                        "Encountered a ContextualLink in a probabilistic chain evaluation.".into(),
                    ));
                }
                _ => {
                    // Other variants are not handled in this mode.
                    return Err(CausalityError(format!(
                        "evaluate_probabilistic_propagation encountered an unhandled effect: {resolved_effect:?}"
                    )));
                }
            }
        }

        Ok(PropagatingEffect::Probabilistic(cumulative_prob))
    }

    /// Evaluates a linear chain of causes that may contain a mix of deterministic and
    /// probabilistic effects, aggregating them into a final effect.
    ///
    /// # Arguments
    /// * `effect` - A single `PropagatingEffect` object that all causes will use.
    ///
    /// # Errors
    /// Returns a `CausalityError` if a `ContextualLink` is encountered.
    fn evaluate_mixed_propagation(
        &self,
        effect: &PropagatingEffect,
    ) -> Result<PropagatingEffect, CausalityError> {
        // The chain starts as deterministically true. It can transition to probabilistic.
        let mut aggregated_effect = PropagatingEffect::Deterministic(true);

        for cause in self.get_all_items() {
            let current_effect = cause.evaluate(effect)?;

            let current_effect = match current_effect {
                PropagatingEffect::RelayTo(target_id, inner_effect) => {
                    let target_causaloid = self
                        .get_item_by_id(target_id as IdentificationValue)
                        .ok_or_else(|| {
                            CausalityError(format!(
                                "Relay target causaloid with ID {target_id} not found."
                            ))
                        })?;
                    target_causaloid.evaluate(&inner_effect)?
                }
                _ => current_effect,
            };

            // Update the aggregated effect based on the current effect.
            aggregated_effect = match (aggregated_effect, current_effect) {
                // Halting takes precedence over everything.
                (_, PropagatingEffect::Halting) => return Ok(PropagatingEffect::Halting),

                // Deterministic false breaks the chain.
                (_, PropagatingEffect::Deterministic(false)) => {
                    return Ok(PropagatingEffect::Deterministic(false));
                }

                // ContextualLink is invalid in this context.
                (_, PropagatingEffect::ContextualLink(_, _)) => {
                    return Err(CausalityError(
                        "Encountered a ContextualLink in a mixed-chain evaluation.".into(),
                    ));
                }

                // If the chain is deterministic and the new effect is true, it remains deterministic true.
                (
                    PropagatingEffect::Deterministic(true),
                    PropagatingEffect::Deterministic(true),
                ) => PropagatingEffect::Deterministic(true),

                // If the chain is deterministic and the new effect is probabilistic, the chain becomes probabilistic.
                (PropagatingEffect::Deterministic(true), PropagatingEffect::Probabilistic(p)) => {
                    PropagatingEffect::Probabilistic(p)
                }

                // If the chain is already probabilistic and the new effect is true, the probability is unchanged.
                (PropagatingEffect::Probabilistic(p), PropagatingEffect::Deterministic(true)) => {
                    PropagatingEffect::Probabilistic(p)
                }

                // If the chain is probabilistic and the new effect is also probabilistic, multiply them.
                (PropagatingEffect::Probabilistic(p1), PropagatingEffect::Probabilistic(p2)) => {
                    PropagatingEffect::Probabilistic(p1 * p2)
                }

                // Other combinations should not be possible due to the guards above.
                (agg, curr) => {
                    return Err(CausalityError(format!(
                        "Unhandled effect combination in mixed chain: Agg: {agg:?}, Curr: {curr:?}"
                    )));
                }
            };
        }

        Ok(aggregated_effect)
    }

    /// Generates an explanation by concatenating the `explain()` text of all causes.
    ///
    /// Each explanation is formatted and separated by newlines.
    /// It gracefully handles errors from individual `explain` calls by inserting
    /// a placeholder error message.
    fn explain(&self) -> Result<String, CausalityError> {
        if self.is_empty() {
            return Err(CausalityError::new(
                "Causal Collection is empty".to_string(),
            ));
        }

        let mut explanation = String::new();
        for cause in self.get_all_items() {
            let cause_explanation = match cause.explain() {
                Ok(s) => s,
                Err(e) => {
                    return Err(CausalityError(format!(
                        "[Error explaining cause {} ('{}')]",
                        cause.id(),
                        e
                    )));
                }
            };

            explanation.push('\n');
            explanation.push_str(format!(" * {cause_explanation}").as_str());
            explanation.push('\n');
        }
        Ok(explanation)
    }
}
