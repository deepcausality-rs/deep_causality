/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    AggregateLogic, Causable, CausalityError, IdentificationValue, NumericalValue,
    PropagatingEffect,
};

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
        _logic: &AggregateLogic,
    ) -> Result<PropagatingEffect, CausalityError> {
        for cause in self.get_all_items() {
            let effect = cause.evaluate(effect)?;

            // This function enforces a strict deterministic contract.
            match effect {
                PropagatingEffect::Deterministic(true) => {
                    // The link is active, continue to the next one.
                    continue;
                }
                PropagatingEffect::Deterministic(false) => {
                    // The chain is deterministically false b/c on causaloid evaluates to false. This is a valid final outcome.
                    return Ok(PropagatingEffect::Deterministic(false));
                }
                _ => {
                    // Any other effect type is a contract violation for this function.
                    return Err(CausalityError(format!(
                        "evaluate_deterministic_propagation encountered a non-deterministic effect: {effect:?}. Only Deterministic effects are allowed."
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
        _logic: &AggregateLogic,
    ) -> Result<PropagatingEffect, CausalityError> {
        // 1.0 is the multiplicative identity for cumulative probability.
        let mut cumulative_prob: NumericalValue = 1.0;

        for cause in self.get_all_items() {
            let effect = cause.evaluate(effect)?;

            match effect {
                PropagatingEffect::Probabilistic(p) | PropagatingEffect::Numerical(p) => {
                    cumulative_prob *= p;
                }

                _ => {
                    // Other variants are not handled in this mode.
                    return Err(CausalityError(format!(
                        "evaluate_probabilistic_propagation encountered a non-probabilistic effect: {effect:?}. Only probabilistic or numerical effects are allowed."
                    )));
                }
            }
        }

        // Convert final probability to a deterministic outcome based on a threshold.
        Ok(PropagatingEffect::Probabilistic(cumulative_prob))
    }

    /// Evaluates a linear chain of causes that may contain a mix of deterministic and
    /// probabilistic effects, aggregating them into a final deterministic outcome.
    ///
    /// This method converts all effects (`Deterministic`, `Probabilistic`, `Numerical`)
    /// into a numerical value (where true=1.0, false=0.0) and aggregates them by
    /// multiplication. The final cumulative probability is then compared against a
    /// threshold (0.5) to produce a final `Deterministic(true)` or `Deterministic(false)`.
    ///
    /// This approach is robust, order-independent, and provides a consistent result.
    ///
    /// # Arguments
    /// * `effect` - A single `PropagatingEffect` object that all causes will use.
    ///
    /// # Errors
    /// Returns a `CausalityError` if a `ContextualLink` is encountered, as it cannot be
    /// converted to a numerical probability.
    fn evaluate_mixed_propagation(
        &self,
        effect: &PropagatingEffect,
        _logic: &AggregateLogic,
    ) -> Result<PropagatingEffect, CausalityError> {
        // Start with 1.0, the multiplicative identity, to aggregate all effects numerically.
        let mut cumulative_prob: NumericalValue = 1.0;

        for cause in self.get_all_items() {
            let current_effect = cause.evaluate(effect)?;

            // Convert every effect to a numerical probability to ensure consistent, order-independent aggregation.
            let current_prob = match current_effect {
                PropagatingEffect::Deterministic(true) => 1.0,
                PropagatingEffect::Deterministic(false) => 0.0,
                PropagatingEffect::Probabilistic(p) => p,
                PropagatingEffect::Numerical(p) => p,
                // .
                _ => {
                    // Other variants are not handled in this mode.
                    return Err(CausalityError(format!(
                        "evaluate_mixed_propagation encountered an unsupported effect: {effect:?}. Only probabilistic, deterministic, or numerical effects are allowed."
                    )));
                }
            };

            cumulative_prob *= current_prob;
        }

        // Convert the final aggregated probability to a deterministic outcome based on a standard threshold.
        if cumulative_prob > 0.5 {
            Ok(PropagatingEffect::Deterministic(true))
        } else {
            Ok(PropagatingEffect::Deterministic(false))
        }
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
