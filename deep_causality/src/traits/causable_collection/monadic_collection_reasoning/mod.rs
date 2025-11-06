/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    AggregateLogic, Causable, CausableCollectionAccessor, CausalMonad, CausalityError,
    MonadicCausable, NumericalValue, PropagatingEffect,
};

/// Provides default implementations for monadic reasoning over collections of `MonadicCausable` items.
///
/// Any collection type that implements the basic accessor methods (`len`, `is_empty`,
/// `to_vec`, `get_all_items`) and `MonadicCausable<CausalMonad>` will automatically gain a suite of useful default
/// methods for inspecting the collective state of its `MonadicCausable` elements.
mod monadic_collection_reasoning_default;

pub trait MonadicCausableCollection<T>: CausableCollectionAccessor<T>
where
    T: MonadicCausable<CausalMonad> + Causable,
{
    /// Evaluates a collection of `MonadicCausable` items, aggregating their monadic effects.
    ///
    /// # Arguments
    /// * `incoming_effect` - A `PropagatingEffect` to be passed to each `MonadicCausable` item.
    ///
    /// # Returns
    /// A `PropagatingEffect` representing the aggregated monadic effect of the collection.
    ///
    /// # Arguments
    /// * `incoming_effect` - A `PropagatingEffect` to be passed to each `MonadicCausable` item.
    /// * `logic` - The `AggregateLogic` (e.g., `All`, `Any`, `None`, `Some(k)`)
    ///   that defines how the boolean results are combined into a final `Deterministic` outcome.
    /// * `threshold` - A `NumericalValue` used to convert the final aggregated uncertainty
    ///   into a `Deterministic` outcome (e.g., if aggregated uncertainty > threshold, then true).
    ///
    /// # Errors
    /// Returns a `CausalityError` if any `MonadicCausable` item returns a non-deterministic effect.
    fn evaluate_collection(
        &self,
        incoming_effect: &PropagatingEffect,
        logic: &AggregateLogic,
        threshold: NumericalValue,
    ) -> PropagatingEffect {
        monadic_collection_reasoning_default::_evaluate_monadic_collection_logic(
            self.get_all_items(),
            incoming_effect,
            logic,
            threshold,
        )
    }
}
