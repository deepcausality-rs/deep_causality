/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalityError, Causaloid};
use deep_causality_core::{EffectValue, PropagatingEffect};
use deep_causality_haft::LogAddEntry;
use std::fmt::Debug;

/// Represents the execution step in a `Causaloid::Singleton` monadic chain.
///
/// This function executes the core causal function (`causal_fn` or `context_causal_fn`)
/// of the causaloid.
///
/// # Arguments
/// * `input`: The specific input value of type `I`.
/// * `causaloid`: A reference to the causaloid containing the function to execute.
///
/// # Returns
/// A `PropagatingEffect` containing either the resulting output
/// value of type `O` or an error.
pub(super) fn execute_causal_logic<I, O, PS, C>(
    input: I,
    causaloid: &Causaloid<I, O, PS, C>,
) -> PropagatingEffect<O>
where
    I: Default + Clone,
    O: Default + Clone + Debug,
    PS: Default + Clone,
    C: Clone,
{
    if let Some(context_fn) = &causaloid.context_causal_fn {
        if let Some(context) = causaloid.context.as_ref() {
            // context_fn signature: fn(EffectValue<I>, PS, Option<C>) -> PropagatingProcess<O, PS, C>
            // We invoke it with default state and the context.
            // The result is PropagatingProcess<O, PS, C>.
            // We need to convert it to PropagatingEffect<O>.

            let ev = EffectValue::from(input);
            let process = context_fn(ev, PS::default(), Some(context.clone()));

            // Convert PropagatingProcess to PropagatingEffect, preserving logs.
            // Convert PropagatingProcess to PropagatingEffect, preserving logs.
            let mut effect = if let Some(error) = process.error {
                PropagatingEffect::from_error(error)
            } else if process.value.is_none() {
                PropagatingEffect::from_error(CausalityError(
                    deep_causality_core::CausalityErrorEnum::Custom(
                        "execute_causal_logic: context_fn returned None value and no error".into(),
                    ),
                ))
            } else {
                PropagatingEffect::from_effect_value(process.value)
            };
            effect.logs = process.logs;
            effect
        } else {
            PropagatingEffect::from_error(CausalityError(
                deep_causality_core::CausalityErrorEnum::Custom(
                    "Causaloid::evaluate: context is None".into(),
                ),
            ))
        }
    } else if let Some(causal_fn) = &causaloid.causal_fn {
        causal_fn(input)
    } else {
        let err_msg = format!(
            "Causaloid {} is missing both causal_fn and context_causal_fn",
            causaloid.id
        );
        PropagatingEffect::from_error(CausalityError(
            deep_causality_core::CausalityErrorEnum::Custom(err_msg),
        ))
    }
}

/// Logs the input to a causaloid.
///
/// # Arguments
/// * `input`: The input value of type `I`.
/// * `id`: The unique identifier of the causaloid.
///
/// # Returns
/// A `PropagatingEffect` containing the input value and a log entry recording it.
pub(super) fn log_input<I>(input: I, id: u64) -> PropagatingEffect<I>
where
    I: Debug + Clone + Default,
{
    let mut effect = PropagatingEffect::pure(input.clone());
    // Format must match expectation: "Causaloid {}: Incoming effect: {:?}"
    let ev = EffectValue::from(input);
    effect
        .logs
        .add_entry(&format!("Causaloid {}: Incoming effect: {:?}", id, ev));
    effect
}

/// Logs the output from a causaloid.
///
/// # Arguments
/// * `output`: The output value of type `O`.
/// * `id`: The unique identifier of the causaloid.
///
/// # Returns
/// A `PropagatingEffect` containing the output value and a log entry recording it.
pub(super) fn log_output<O>(output: O, id: u64) -> PropagatingEffect<O>
where
    O: Debug + Clone + Default,
{
    let mut effect = PropagatingEffect::pure(output.clone());
    // Format must match expectation: "Causaloid {}: Outgoing effect: {:?}"
    let ev = EffectValue::from(output);
    effect
        .logs
        .add_entry(&format!("Causaloid {}: Outgoing effect: {:?}", id, ev));
    effect
}
