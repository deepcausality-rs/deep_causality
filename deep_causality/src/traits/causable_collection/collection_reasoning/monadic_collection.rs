/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    AggregateLogic, Causable, CausableCollectionAccessor, CausalMonad, CausalityError,
    MonadicCausable, NumericalValue, PropagatingEffect, monadic_collection_utils,
};
use deep_causality_core::{CausalityErrorEnum, EffectValue};
use deep_causality_haft::*;

pub trait MonadicCausableCollection<I, O, T>: CausableCollectionAccessor<I, O, T>
where
    T: MonadicCausable<I, O> + Causable,
    O: monadic_collection_utils::Aggregatable
        + Clone
        + Default
        + Send
        + Sync
        + 'static
        + std::fmt::Debug,
{
    /// Evaluates a collection of `MonadicCausable` items, aggregating their monadic effects.
    ///
    /// This method provides a single, unified entry point for reasoning over a collection of causable items.
    /// It monadically evaluates each item and then uses a type-aware aggregation strategy to combine
    /// the results.
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
    ///
    /// # Errors
    ///
    /// Returns a `PropagatingEffect` containing a `CausalityError` if:
    /// * The collection is empty.
    /// * An item's `evaluate` method returns an error.
    fn evaluate_collection(
        &self,
        incoming_effect: &PropagatingEffect<I>,
        logic: &AggregateLogic,
        threshold_value: Option<NumericalValue>,
    ) -> PropagatingEffect<O> {
        let items = self.get_all_items();

        if items.is_empty() {
            let err = CausalityError(CausalityErrorEnum::Custom(
                "Cannot evaluate an empty collection".to_string(),
            ));
            return PropagatingEffect::from_error(err);
        }
        // 1. Monadic fold to collect all effects.
        // We start with a pure effect containing an empty vector of EffectValue<O>.
        let initial_effect: PropagatingEffect<Vec<EffectValue<O>>> = CausalMonad::pure(Vec::new());

        let final_effect = items.into_iter().fold(initial_effect, |acc_effect, item| {
            acc_effect.bind(|acc_values_effect_value, _, _| {
                // acc_values_effect_value is EffectValue<Vec<EffectValue<O>>>
                // We need to extract the Vec from it
                let mut acc_values = match acc_values_effect_value.into_value() {
                    Some(v) => v,
                    None => {
                        let err = CausalityError(CausalityErrorEnum::Custom(
                            "Failed to extract accumulated values during collection evaluation"
                                .to_string(),
                        ));
                        return PropagatingEffect::from_error(err);
                    }
                };
                let item_effect = item.evaluate(incoming_effect); // item_effect is PropagatingEffect<O>
                // Use bind to extract the value and transform the result
                item_effect.bind(|item_value, _, _| {
                    // item_value is EffectValue<O>
                    acc_values.push(item_value);
                    CausalMonad::pure(acc_values)
                })
            })
        });

        // 2. Bind the final aggregation logic.
        final_effect.bind(|effect_values_effect_value, _, _| {
            // effect_values_effect_value is EffectValue<Vec<EffectValue<O>>>
            // We need to extract the Vec from it
            let effect_values = match effect_values_effect_value.into_value() {
                Some(v) => v,
                None => {
                    let err = CausalityError(CausalityErrorEnum::Custom(
                        "No effect values collected".to_string(),
                    ));
                    return PropagatingEffect::from_error(err);
                }
            };
            // 3. Delegate to the robust aggregation helper.
            match monadic_collection_utils::aggregate_effects(
                &effect_values,
                logic,
                threshold_value,
            ) {
                Ok(aggregated_value) => {
                    // aggregated_value is EffectValue<O>.
                    // We need to return PropagatingEffect<O>.
                    // If aggregated_value is Value(v), we return pure(v).
                    match aggregated_value {
                        EffectValue::Value(v) => CausalMonad::pure(v),
                        _ => {
                            // This case shouldn't happen for aggregation result usually.
                            // We can construct PropagatingEffect with this value.
                            let mut eff = CausalMonad::pure(O::default());
                            eff.value = aggregated_value;
                            eff
                        }
                    }
                }
                Err(e) => PropagatingEffect::from_error(e),
            }
        })
    }
}
