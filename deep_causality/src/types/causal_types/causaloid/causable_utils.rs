/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    CausalEffectLog, CausalMonad, CausalPropagatingEffect, CausalityError, Causaloid, Datable,
    EffectValue, IntoEffectValue, SpaceTemporal, Spatial, Symbolic, Temporal,
};
use deep_causality_haft::MonadEffect3;

/// Represents the first step in a `Causaloid::Singleton` monadic chain.
///
/// This function attempts to convert a generic `EffectValue` into the specific
/// input type `I` required by the causaloid.
///
/// # Arguments
/// * `effect_val`: The `EffectValue` from the previous step.
/// * `id`: The ID of the causaloid for logging purposes.
///
/// # Returns
/// A new `CausalPropagatingEffect` containing either the successfully converted
/// value of type `I` or an error.
///
/// # Log Provenance
/// This effect only contains logs generated during this specific operation.
/// It is the responsibility of the calling `bind` function to merge these logs
/// with the preceding log history.
pub(super) fn convert_input<I>(
    effect_val: EffectValue,
    id: u64,
) -> CausalPropagatingEffect<I, CausalityError, CausalEffectLog>
where
    I: IntoEffectValue + Default,
{
    match I::try_from_effect_value(effect_val) {
        Ok(val) => CausalMonad::pure(val),
        Err(e) => CausalPropagatingEffect {
            value: I::default(),
            error: Some(e.clone()),
            logs: format!("Causaloid {}: Input conversion failed: {}", id, e).into(),
        },
    }
}

/// Represents the second step in a `Causaloid::Singleton` monadic chain.
///
/// This function executes the core causal function (`causal_fn` or `context_causal_fn`)
/// of the causaloid.
///
/// # Arguments
/// * `input`: The specific input value of type `I`, successfully converted from the previous step.
/// * `causaloid`: A reference to the causaloid containing the function to execute.
///
/// # Returns
/// A new `CausalPropagatingEffect` containing either the resulting output
/// value of type `O` or an error.
///
/// # Log Provenance
/// This effect only contains logs generated during this specific operation.
/// It is the responsibility of the calling `bind` function to merge these logs
/// with the preceding log history.
pub(super) fn execute_causal_logic<I, O, D, S, T, ST, SYM, VS, VT>(
    input: I,
    causaloid: &Causaloid<I, O, D, S, T, ST, SYM, VS, VT>,
) -> CausalPropagatingEffect<O, CausalityError, CausalEffectLog>
where
    I: IntoEffectValue + Default,
    O: IntoEffectValue + Default,
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    let result = if let Some(context_fn) = causaloid.context_causal_fn {
        if let Some(context) = causaloid.context.as_ref() {
            context_fn(input, context)
        } else {
            Err(CausalityError(
                "Causaloid::evaluate: context is None".into(),
            ))
        }
    } else if let Some(causal_fn) = causaloid.causal_fn {
        causal_fn(input)
    } else {
        let err_msg = format!(
            "Causaloid {} is missing both causal_fn and context_causal_fn",
            causaloid.id
        );
        Err(CausalityError(err_msg))
    };

    match result {
        Ok(causal_fn_output) => {
            // Create a new effect with the output value and the log from the causal function.
            CausalPropagatingEffect {
                value: causal_fn_output.output,
                error: None,
                logs: causal_fn_output.log,
            }
        }
        Err(e) => CausalPropagatingEffect {
            value: O::default(),
            error: Some(e.clone()),
            logs: format!("Causaloid {}: Causal function failed: {}", causaloid.id, e).into(),
        },
    }
}

/// Represents the final step in a `Causaloid::Singleton` monadic chain.
///
/// This function converts the specific output type `O` from the causal function
/// back into a generic `EffectValue`.
///
/// # Arguments
/// * `output`: The specific output value of type `O` from the previous step.
/// * `id`: The ID of the causaloid for logging purposes.
///
/// # Returns
/// A new `CausalPropagatingEffect` containing the final `EffectValue`.
///
/// # Log Provenance
/// This effect contains the log message for this specific operation. It is the
/// responsibility of the calling `bind` function to merge this log with the
/// preceding log history.
pub(super) fn convert_output<O>(
    output: O,
    id: u64,
) -> CausalPropagatingEffect<EffectValue, CausalityError, CausalEffectLog>
where
    O: IntoEffectValue,
{
    let effect_value = output.into_effect_value();

    let mut monad = CausalMonad::pure(effect_value.clone());
    monad.logs.add_entry(&format!(
        "Causaloid {}: Outgoing effect: {:?}",
        id, effect_value
    ));
    monad
}
