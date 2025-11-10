/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    AggregateLogic, Causable, CausableCollectionAccessor, CausalMonad, CausalityError,
    CausaloidRegistry, MonadicCausable, NumericalValue, PropagatingEffect,
    monadic_collection_utils,
};
use deep_causality_haft::*;

pub trait MonadicCausableCollection<T>: CausableCollectionAccessor<T>
where
    T: MonadicCausable<CausalMonad> + Causable,
{
    /// Evaluates a collection of `MonadicCausable` items, aggregating their monadic effects.
    ///
    /// This method provides a single, unified entry point for reasoning over a collection of causable items.
    /// It monadically evaluates each item and then uses a type-aware aggregation strategy to combine
    /// the results, preserving as much information as possible (e.g., returning a `Probabilistic` effect
    /// if the collection contains probabilities, rather than collapsing it to a boolean).
    ///
    /// # Arguments
    ///
    /// * `incoming_effect` - A `PropagatingEffect` to be passed to each `MonadicCausable` item.
    /// * `logic` - The `AggregateLogic` (e.g., `All`, `Any`, `None`, `Some(k)`) that defines how the results are combined.
    /// * `threshold_value` - An optional `NumericalValue` used for comparisons (e.g., converting a probability to a boolean).
    ///   It is required for some aggregation strategies.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect` representing the aggregated monadic effect of the collection.
    /// The `EffectValue` inside will be of the "highest" type found during aggregation
    /// (e.g., `UncertainBool` > `Probabilistic` > `Deterministic`).
    ///
    /// # Errors
    ///
    /// Returns a `PropagatingEffect` containing a `CausalityError` if:
    /// * The collection is empty.
    /// * An item's `reason` method returns an error.
    /// * The collected effects are of incompatible types for the chosen aggregation strategy.
    ///
    fn evaluate_collection(
        &self,
        registry: &CausaloidRegistry,
        incoming_effect: &PropagatingEffect,
        logic: &AggregateLogic,
        threshold_value: Option<NumericalValue>,
    ) -> PropagatingEffect {
        let items = self.get_all_items();

        if items.is_empty() {
            let err = CausalityError("Cannot evaluate an empty collection".to_string());
            return PropagatingEffect::from_error(err);
        }

        // 1. Monadic fold to collect all effects.
        let initial_effect = CausalMonad::pure(Vec::new());

        let final_effect = items.into_iter().fold(initial_effect, |acc_effect, item| {
            CausalMonad::bind(acc_effect, |mut acc_values| {
                let item_effect = item.evaluate(registry, incoming_effect);
                CausalMonad::bind(item_effect, |item_value| {
                    acc_values.push(item_value);
                    CausalMonad::pure(acc_values.clone())
                })
            })
        });

        // 2. Bind the final aggregation logic.
        CausalMonad::bind(final_effect, |effect_values| {
            // 3. Delegate to the robust aggregation helper.
            match monadic_collection_utils::aggregate_effects(effect_values, logic, threshold_value)
            {
                Ok(aggregated_value) => CausalMonad::pure(aggregated_value),
                Err(e) => PropagatingEffect::from_error(e),
            }
        })
    }
}
