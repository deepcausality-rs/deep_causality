/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    AggregateLogic, Causable, CausalEffectLog, CausalMonad, CausalityError, EffectValue,
    MonadicCausable, NumericalValue, PropagatingEffect,
};

pub fn _evaluate_monadic_collection_logic<T>(
    items: Vec<&T>,
    incoming_effect: &PropagatingEffect,
    logic: &AggregateLogic,
    threshold: NumericalValue,
) -> PropagatingEffect
where
    T: MonadicCausable<CausalMonad> + Causable,
{
    let mut collected_effects: Vec<PropagatingEffect> = Vec::new();
    let mut collected_logs: Vec<CausalEffectLog> = Vec::new();
    let mut first_error: Option<CausalityError> = None;

    for item in items {
        let effect = item.evaluate_monadic(incoming_effect.clone());

        if first_error.is_none() {
            first_error = effect.error.clone();
        }
        collected_logs.extend(effect.logs.clone());
        collected_effects.push(effect);
    }

    // If any error occurred, return it immediately.
    if let Some(err) = first_error {
        return PropagatingEffect {
            value: EffectValue::None,
            error: Some(err),
            logs: collected_logs,
        };
    }

    // Aggregate the values based on AggregateLogic and threshold
    let aggregated_value = aggregate_effect_values(
        collected_effects.iter().map(|e| &e.value).collect(),
        logic,
        threshold,
    )
    .unwrap();

    PropagatingEffect {
        value: aggregated_value,
        error: None,
        logs: collected_logs,
    }
}

// Helper function to aggregate EffectValues
fn aggregate_effect_values(
    values: Vec<&EffectValue>,
    logic: &AggregateLogic,
    threshold: NumericalValue,
) -> Result<EffectValue, CausalityError> {
    let mut bool_values = Vec::new();
    for value in values {
        bool_values.push(effect_value_to_bool(value, threshold)?);
    }

    let result_bool = match logic {
        AggregateLogic::All => bool_values.iter().all(|&b| b),
        AggregateLogic::Any => bool_values.iter().any(|&b| b),
        AggregateLogic::None => bool_values.iter().all(|&b| !b),
        AggregateLogic::Some(k) => bool_values.iter().filter(|&&b| b).count() >= *k,
    };

    Ok(EffectValue::Deterministic(result_bool))
}

// Helper function to convert EffectValue to bool
fn effect_value_to_bool(
    value: &EffectValue,
    threshold: NumericalValue,
) -> Result<bool, CausalityError> {
    match value {
        EffectValue::Deterministic(b) => Ok(*b),
        EffectValue::Probabilistic(p) => Ok(*p >= threshold),
        EffectValue::Numerical(n) => Ok(*n >= threshold),
        EffectValue::Number(n) => match n {
            crate::NumericValue::F64(f) => Ok(*f >= threshold),
            _ => Err(CausalityError::new(
                "Cannot convert non-f64 NumericValue to bool for aggregation".to_string(),
            )),
        },
        EffectValue::UncertainBool(ub) => Ok(ub.value()),
        _ => Err(CausalityError::new(
            "Unsupported EffectValue for boolean aggregation".to_string(),
        )),
    }
}
