/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod causable_reasoning_deterministic;
mod causable_reasoning_mixed;
mod causable_reasoning_probabilistic;

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    AggregateLogic, Causable, CausableCollectionAccessor, CausalityError, IdentificationValue,
    NumericalValue, PropagatingEffect,
};

/// Provides default implementations for reasoning over collections of `Causable` items.
///
/// Any collection type that implements the basic accessor methods (`len`, `is_empty`,
/// `to_vec`, `get_all_items`) will automatically gain a suite of useful default
/// methods for inspecting the collective state of its `Causable` elements.
pub trait CausableCollectionReasoning<T>: CausableCollectionAccessor<T>
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

    /// Returns a reference to a `Causable` item by its ID, if found.
    fn get_item_by_id(&self, id: IdentificationValue) -> Option<&T>;

    //
    // Default implementations for all other methods are provided below.
    //

    /// Evaluates a collection of `Causable` items, aggregating their deterministic
    /// boolean outcomes (`true`/`false`) based on a specified `AggregateLogic`.
    ///
    /// This function requires that every item in the collection evaluates to a
    /// `PropagatingEffect::Deterministic` variant. If any item returns a different
    /// variant, the function will fail with a `CausalityError`, ensuring strict
    /// type checking at runtime.
    ///
    /// # Arguments
    /// * `effect` - A `PropagatingEffect` to be passed to each `Causable` item.
    /// * `logic` - The `AggregateLogic` (e.g., `All`, `Any`, `None`, `Some(k)`)
    ///   that defines how the boolean results are combined into a final `Deterministic` outcome.
    ///
    /// # Errors
    /// Returns a `CausalityError` if any `Causable` item returns a non-deterministic effect.
    fn evaluate_deterministic(
        &self,
        effect: &PropagatingEffect,
        logic: &AggregateLogic,
    ) -> Result<PropagatingEffect, CausalityError> {
        // Delegate to private impl in causable_reasoning_deterministic
        causable_reasoning_deterministic::_evaluate_deterministic_logic(
            self.get_all_items(),
            effect,
            logic,
        )
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
    fn evaluate_probabilistic(
        &self,
        effect: &PropagatingEffect,
        logic: &AggregateLogic,
        threshold: NumericalValue,
    ) -> Result<PropagatingEffect, CausalityError> {
        // Delegate to private impl in causable_reasoning_probabilistic
        causable_reasoning_probabilistic::_evaluate_probabilistic_logic(
            self.get_all_items(),
            effect,
            logic,
            threshold,
        )
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
    fn evaluate_mixed(
        &self,
        effect: &PropagatingEffect,
        logic: &AggregateLogic,
        threshold: NumericalValue,
    ) -> Result<PropagatingEffect, CausalityError> {
        // Delegate to private impl in causable_reasoning_mixed
        causable_reasoning_mixed::_evaluate_mixed_logic(
            self.get_all_items(),
            effect,
            logic,
            threshold,
        )
    }
}
