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
        logic: &AggregateLogic,
    ) -> Result<PropagatingEffect, CausalityError> {
        let mut resolved_effects = Vec::new();

        for cause in self.get_all_items() {
            let current_effect = cause.evaluate(effect)?;

            let resolved_effect = match current_effect {
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
            resolved_effects.push(resolved_effect);
        }

        match logic {
            AggregateLogic::All => {
                let mut last_true_effect = PropagatingEffect::Deterministic(true);
                for res_effect in resolved_effects {
                    match res_effect {
                        PropagatingEffect::Deterministic(true) => {
                            last_true_effect = res_effect;
                        }
                        PropagatingEffect::Deterministic(false) => {
                            return Ok(PropagatingEffect::Deterministic(false));
                        }
                        PropagatingEffect::Halting => {
                            return Ok(PropagatingEffect::Halting);
                        }
                        _ => {
                            return Err(CausalityError(format!(
                                "evaluate_deterministic_propagation (All) encountered a non-deterministic effect: {res_effect:?}. Only Deterministic effects are allowed."
                            )));
                        }
                    }
                }
                Ok(last_true_effect)
            }
            AggregateLogic::Any => {
                let mut has_true = false;
                let mut last_effect = PropagatingEffect::Halting; // Default to Halting if no true or false found
                for res_effect in resolved_effects {
                    match res_effect {
                        PropagatingEffect::Deterministic(true) => {
                            has_true = true;
                            last_effect = res_effect;
                        }
                        PropagatingEffect::Deterministic(false) => {
                            last_effect = res_effect;
                        }
                        PropagatingEffect::Halting => {
                            return Ok(PropagatingEffect::Halting);
                        }
                        _ => {
                            return Err(CausalityError(format!(
                                "evaluate_deterministic_propagation (Any) encountered a non-deterministic effect: {res_effect:?}. Only Deterministic effects are allowed."
                            )));
                        }
                    }
                }
                if has_true {
                    Ok(last_effect)
                } else {
                    Ok(PropagatingEffect::Deterministic(false))
                }
            }
            AggregateLogic::None => {
                let mut all_false = true;
                let mut last_false_effect = PropagatingEffect::Deterministic(false);
                for res_effect in resolved_effects {
                    match res_effect {
                        PropagatingEffect::Deterministic(true) => {
                            all_false = false;
                            break;
                        }
                        PropagatingEffect::Deterministic(false) => {
                            last_false_effect = res_effect;
                        }
                        PropagatingEffect::None => {
                            return Ok(PropagatingEffect::None);
                        }
                        PropagatingEffect::Halting => {
                            return Ok(PropagatingEffect::Halting);
                        }
                        _ => {
                            return Err(CausalityError(format!(
                                "evaluate_deterministic_propagation (None) encountered a non-deterministic effect: {res_effect:?}. Only Deterministic effects are allowed."
                            )));
                        }
                    }
                }
                if all_false {
                    Ok(PropagatingEffect::Deterministic(true))
                } else {
                    Ok(PropagatingEffect::Deterministic(false))
                }
            }
            AggregateLogic::Some(k) => {
                let mut true_count = 0;
                let mut last_true_effect = PropagatingEffect::Halting;
                for res_effect in resolved_effects {
                    match res_effect {
                        PropagatingEffect::Deterministic(true) => {
                            true_count += 1;
                            last_true_effect = res_effect;
                        }
                        PropagatingEffect::Deterministic(false) => {}
                        PropagatingEffect::Halting => {
                            return Ok(PropagatingEffect::Halting);
                        }
                        _ => {
                            return Err(CausalityError(format!(
                                "evaluate_deterministic_propagation (Some) encountered a non-deterministic effect: {res_effect:?}. Only Deterministic effects are allowed."
                            )));
                        }
                    }
                }
                if true_count >= *k {
                    Ok(PropagatingEffect::Deterministic(true))
                } else {
                    Ok(PropagatingEffect::Deterministic(false))
                }
            }
        }
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
        logic: &AggregateLogic,
    ) -> Result<PropagatingEffect, CausalityError> {
        let mut resolved_effects = Vec::new();

        for cause in self.get_all_items() {
            let current_effect = cause.evaluate(effect)?;

            let resolved_effect = match current_effect {
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
            resolved_effects.push(resolved_effect);
        }

        match logic {
            AggregateLogic::All => {
                let mut cumulative_prob: NumericalValue = 1.0;
                for res_effect in resolved_effects {
                    match res_effect {
                        PropagatingEffect::None => {
                            return Ok(PropagatingEffect::None);
                        }
                        PropagatingEffect::Halting => {
                            return Ok(PropagatingEffect::Halting);
                        }

                        PropagatingEffect::Probabilistic(p) => {
                            cumulative_prob *= p;
                            if cumulative_prob == 0.0 {
                                return Ok(PropagatingEffect::Probabilistic(0.0));
                            }
                        }
                        PropagatingEffect::Deterministic(true) => {
                            // Equivalent to multiplying by 1.0
                        }
                        PropagatingEffect::Deterministic(false) => {
                            return Ok(PropagatingEffect::Probabilistic(0.0));
                        }

                        _ => {
                            return Err(CausalityError(format!(
                                "evaluate_probabilistic_propagation (All) encountered an unhandled effect: {res_effect:?}"
                            )));
                        }
                    }
                }
                Ok(PropagatingEffect::Probabilistic(cumulative_prob))
            }
            AggregateLogic::Any => {
                for res_effect in resolved_effects {
                    match res_effect {
                        PropagatingEffect::Probabilistic(p) => {
                            if p > 0.0 {
                                return Ok(PropagatingEffect::Probabilistic(p));
                            }
                        }
                        PropagatingEffect::Deterministic(true) => {
                            return Ok(PropagatingEffect::Deterministic(true));
                        }
                        PropagatingEffect::Deterministic(false) => {}
                        PropagatingEffect::Halting => {
                            return Ok(PropagatingEffect::Halting);
                        }
                        _ => {
                            return Err(CausalityError(format!(
                                "evaluate_probabilistic_propagation (Any) encountered an unhandled effect: {res_effect:?}"
                            )));
                        }
                    }
                }
                Ok(PropagatingEffect::Probabilistic(0.0))
            }
            AggregateLogic::None => {
                for res_effect in resolved_effects {
                    match res_effect {
                        PropagatingEffect::Probabilistic(p) => {
                            if p > 0.0 {
                                return Ok(PropagatingEffect::Probabilistic(0.0));
                            }
                        }
                        PropagatingEffect::Deterministic(true) => {
                            return Ok(PropagatingEffect::Probabilistic(0.0));
                        }
                        PropagatingEffect::Deterministic(false) => {}
                        PropagatingEffect::Halting => {
                            return Ok(PropagatingEffect::Halting);
                        }
                        _ => {
                            return Err(CausalityError(format!(
                                "evaluate_probabilistic_propagation (None) encountered an unhandled effect: {res_effect:?}"
                            )));
                        }
                    }
                }
                Ok(PropagatingEffect::Probabilistic(1.0))
            }
            AggregateLogic::Some(k) => {
                let mut success_count = 0;
                let mut last_successful_effect = PropagatingEffect::Probabilistic(0.0);
                for res_effect in resolved_effects {
                    match res_effect {
                        PropagatingEffect::Probabilistic(p) => {
                            if p > 0.0 {
                                success_count += 1;
                                last_successful_effect = res_effect;
                            }
                        }
                        PropagatingEffect::Deterministic(true) => {
                            success_count += 1;
                            last_successful_effect = res_effect;
                        }
                        PropagatingEffect::Deterministic(false) => {}
                        PropagatingEffect::Halting => {
                            return Ok(PropagatingEffect::Halting);
                        }
                        _ => {
                            return Err(CausalityError(format!(
                                "evaluate_probabilistic_propagation (Some) encountered an unhandled effect: {res_effect:?}"
                            )));
                        }
                    }
                }
                if success_count >= *k {
                    Ok(last_successful_effect)
                } else {
                    Ok(PropagatingEffect::Probabilistic(0.0))
                }
            }
        }
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
        logic: &AggregateLogic,
    ) -> Result<PropagatingEffect, CausalityError> {
        let mut resolved_effects = Vec::new();

        for cause in self.get_all_items() {
            let current_effect = cause.evaluate(effect)?;

            let resolved_effect = match current_effect {
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
            resolved_effects.push(resolved_effect);
        }

        match logic {
            AggregateLogic::All => {
                let mut all_true = true;
                let mut last_successful_effect = PropagatingEffect::Deterministic(true);
                for res_effect in resolved_effects {
                    match res_effect {
                        PropagatingEffect::Deterministic(true) => {
                            last_successful_effect = res_effect;
                        }
                        PropagatingEffect::Probabilistic(p) => {
                            if p > 0.0 {
                                last_successful_effect = res_effect;
                            } else {
                                all_true = false;
                                break;
                            }
                        }
                        PropagatingEffect::Deterministic(false) => {
                            all_true = false;
                            break;
                        }
                        PropagatingEffect::Halting => {
                            return Ok(PropagatingEffect::Halting);
                        }
                        PropagatingEffect::ContextualLink(_, _) => {
                            return Err(CausalityError(
                                "evaluate_mixed_propagation (All) encountered a ContextualLink."
                                    .into(),
                            ));
                        }
                        _ => {
                            return Err(CausalityError(format!(
                                "evaluate_mixed_propagation (All) encountered an unhandled effect: {res_effect:?}"
                            )));
                        }
                    }
                }
                if all_true {
                    Ok(last_successful_effect)
                } else {
                    Ok(PropagatingEffect::Deterministic(false))
                }
            }
            AggregateLogic::Any => {
                let mut any_true = false;
                let mut last_successful_effect = PropagatingEffect::Deterministic(false);
                let mut final_failure_effect: Option<PropagatingEffect> = None;

                for res_effect in resolved_effects {
                    match res_effect {
                        PropagatingEffect::Deterministic(true) => {
                            any_true = true;
                            last_successful_effect = res_effect;
                        }
                        PropagatingEffect::Probabilistic(p) => {
                            if p > 0.0 {
                                any_true = true;
                                last_successful_effect = res_effect;
                            } else {
                                // If a probabilistic 0.0 is encountered, it's a potential failure
                                if final_failure_effect.is_none() {
                                    final_failure_effect =
                                        Some(PropagatingEffect::Probabilistic(0.0));
                                }
                            }
                        }
                        PropagatingEffect::Deterministic(false) => {
                            // If a deterministic false is encountered, it's a potential failure
                            if final_failure_effect.is_none()
                                || matches!(
                                    final_failure_effect,
                                    Some(PropagatingEffect::Probabilistic(_))
                                )
                            {
                                final_failure_effect =
                                    Some(PropagatingEffect::Deterministic(false));
                            }
                        }
                        PropagatingEffect::Halting => {
                            return Ok(PropagatingEffect::Halting);
                        }
                        PropagatingEffect::ContextualLink(_, _) => {
                            return Err(CausalityError(
                                "evaluate_mixed_propagation (Any) encountered a ContextualLink."
                                    .into(),
                            ));
                        }
                        _ => {
                            return Err(CausalityError(format!(
                                "evaluate_mixed_propagation (Any) encountered an unhandled effect: {res_effect:?}"
                            )));
                        }
                    }
                }
                if any_true {
                    Ok(last_successful_effect)
                } else if let Some(failure_effect) = final_failure_effect {
                    Ok(failure_effect)
                } else {
                    Ok(PropagatingEffect::Deterministic(false)) // Default if no true and no specific failure effect
                }
            }
            AggregateLogic::None => {
                let mut any_true = false;
                let mut final_failure_effect: Option<PropagatingEffect> = None;

                for res_effect in resolved_effects {
                    match res_effect {
                        PropagatingEffect::Deterministic(true) => {
                            any_true = true;
                            final_failure_effect = Some(PropagatingEffect::Deterministic(false));
                            break;
                        }
                        PropagatingEffect::Probabilistic(p) => {
                            if p > 0.0 {
                                any_true = true;
                                final_failure_effect = Some(PropagatingEffect::Probabilistic(0.0));
                                break;
                            }
                        }
                        PropagatingEffect::Deterministic(false) => {}
                        PropagatingEffect::Halting => {
                            return Ok(PropagatingEffect::Halting);
                        }
                        PropagatingEffect::ContextualLink(_, _) => {
                            return Err(CausalityError(
                                "evaluate_mixed_propagation (None) encountered a ContextualLink."
                                    .into(),
                            ));
                        }
                        _ => {
                            return Err(CausalityError(format!(
                                "evaluate_mixed_propagation (None) encountered an unhandled effect: {res_effect:?}"
                            )));
                        }
                    }
                }
                if any_true {
                    if let Some(failure_effect) = final_failure_effect {
                        Ok(failure_effect)
                    } else {
                        Ok(PropagatingEffect::Deterministic(false)) // Should not happen if any_true is true
                    }
                } else {
                    Ok(PropagatingEffect::Deterministic(true))
                }
            }
            AggregateLogic::Some(k) => {
                let mut success_count = 0;
                let mut last_successful_effect = PropagatingEffect::Deterministic(false);
                let mut final_failure_effect: Option<PropagatingEffect> = None;

                for res_effect in resolved_effects {
                    match res_effect {
                        PropagatingEffect::Deterministic(true) => {
                            success_count += 1;
                            last_successful_effect = res_effect;
                        }
                        PropagatingEffect::Probabilistic(p) => {
                            if p > 0.0 {
                                success_count += 1;
                                last_successful_effect = res_effect;
                            }
                        }
                        PropagatingEffect::Deterministic(false) => {
                            if final_failure_effect.is_none()
                                || matches!(
                                    final_failure_effect,
                                    Some(PropagatingEffect::Probabilistic(_))
                                )
                            {
                                // Prioritize deterministic false over probabilistic 0.0
                                final_failure_effect =
                                    Some(PropagatingEffect::Deterministic(false));
                            }
                        }
                        PropagatingEffect::Halting => {
                            return Ok(PropagatingEffect::Halting);
                        }
                        PropagatingEffect::ContextualLink(_, _) => {
                            return Err(CausalityError(
                                "evaluate_mixed_propagation (Some) encountered a ContextualLink."
                                    .into(),
                            ));
                        }
                        _ => {
                            return Err(CausalityError(format!(
                                "evaluate_mixed_propagation (Some) encountered an unhandled effect: {res_effect:?}"
                            )));
                        }
                    }
                }
                if success_count >= *k {
                    Ok(last_successful_effect)
                } else if let Some(failure_effect) = final_failure_effect {
                    Ok(failure_effect)
                } else {
                    Ok(PropagatingEffect::Deterministic(false)) // Default if not enough successes and no specific failure effect
                }
            }
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
