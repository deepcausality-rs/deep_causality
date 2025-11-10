/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{AggregateLogic, CausalityError, EffectValue, NumericalValue};
use deep_causality_uncertain::Uncertain;

/// Main dispatcher function for aggregation.
/// It inspects the collected effects and chooses the highest-fidelity aggregation strategy possible.
pub(crate) fn aggregate_effects(
    effects: Vec<EffectValue>,
    logic: &AggregateLogic,
    threshold_value: Option<NumericalValue>,
) -> Result<EffectValue, CausalityError> {
    if effects.is_empty() {
        return Err(CausalityError(
            "Cannot aggregate empty collection".to_string(),
        ));
    }

    // Strategy Selection: Determine the highest-order effect type in the collection.
    let has_uncertain = effects
        .iter()
        .any(|e| e.is_uncertain_bool() || e.is_uncertain_float());
    let has_probabilistic = effects.iter().any(|e| e.is_probabilistic());

    if has_uncertain {
        // If any effect is Uncertain, use the uncertain aggregation strategy.
        aggregate_uncertain(&effects, logic, threshold_value)
    } else if has_probabilistic {
        // Else, if any is Probabilistic, use the probabilistic strategy.
        aggregate_probabilistic(&effects, logic, threshold_value)
    } else {
        // Otherwise, use the strict deterministic strategy.
        aggregate_deterministic(&effects, logic)
    }
}

/// Strategy 1: Deterministic Aggregation.
/// Mirrors the legacy `_evaluate_deterministic_logic`.
fn aggregate_deterministic(
    effects: &[EffectValue],
    logic: &AggregateLogic,
) -> Result<EffectValue, CausalityError> {
    let bools: Result<Vec<bool>, _> = effects.iter().map(|e| match e {
            EffectValue::Deterministic(b) => Ok(*b),
            _ => Err(CausalityError(format!("Deterministic aggregation requires all effects to be Deterministic, but found {:?}", e))),
        }).collect();

    let bools = bools?;
    let final_bool = match logic {
        AggregateLogic::All => bools.iter().all(|&b| b),
        AggregateLogic::Any => bools.iter().any(|&b| b),
        AggregateLogic::None => !bools.iter().any(|&b| b),
        AggregateLogic::Some(k) => bools.iter().filter(|&&b| b).count() >= *k,
    };
    Ok(EffectValue::Deterministic(final_bool))
}

/// Strategy 2: Probabilistic Aggregation.
/// Mirrors the legacy `_evaluate_probabilistic_logic`.
fn aggregate_probabilistic(
    effects: &[EffectValue],
    logic: &AggregateLogic,
    threshold_opt: Option<NumericalValue>,
) -> Result<EffectValue, CausalityError> {
    let threshold = threshold_opt.unwrap_or(0.5); // Default threshold for this strategy.

    let probes: Result<Vec<f64>, _> = effects
        .iter()
        .map(|e| match e {
            EffectValue::Deterministic(b) => Ok(if *b { 1.0 } else { 0.0 }),
            EffectValue::Probabilistic(p) => Ok(*p),
            EffectValue::UncertainBool(ub) => {
                ub.estimate_probability(100).map_err(CausalityError::from)
            } // num_samples
            EffectValue::UncertainFloat(uf) => uf
                .estimate_probability_exceeds(threshold, 100)
                .map_err(CausalityError::from),
            _ => Err(CausalityError(format!(
                "Unsupported type for probabilistic aggregation: {:?}",
                e
            ))),
        })
        .collect();

    let probs = probes?;
    let final_prob = match logic {
        AggregateLogic::All => probs.iter().product(),
        AggregateLogic::Any => 1.0 - probs.iter().map(|p| 1.0 - p).product::<f64>(),
        AggregateLogic::None => 1.0 - (1.0 - probs.iter().map(|p| 1.0 - p).product::<f64>()),
        AggregateLogic::Some(k) => {
            let count = probs.iter().filter(|&&p| p > 0.5).count();
            if count >= *k { 1.0 } else { 0.0 }
        }
    };
    Ok(EffectValue::Probabilistic(final_prob))
}

/// Strategy 3: Uncertain Aggregation.
/// Mirrors the legacy `_evaluate_uncertain_logic`.
fn aggregate_uncertain(
    effects: &[EffectValue],
    logic: &AggregateLogic,
    threshold_opt: Option<NumericalValue>,
) -> Result<EffectValue, CausalityError> {
    let threshold = threshold_opt.ok_or_else(|| {
        CausalityError("Threshold is required for uncertain aggregation".to_string())
    })?;

    let uncertain_bools: Result<Vec<_>, _> = effects
        .iter()
        .map(|e| match e {
            EffectValue::UncertainBool(ub) => Ok(ub.clone()),
            EffectValue::UncertainFloat(uf) => Ok(uf.greater_than(threshold)),
            _ => Err(CausalityError(format!(
                "Unsupported type for uncertain aggregation: {:?}",
                e
            ))),
        })
        .collect();

    let u_bools = uncertain_bools?;
    if u_bools.is_empty() {
        return Err(CausalityError(
            "No uncertain-compatible effects found".to_string(),
        ));
    }

    let final_ubool = match logic {
        AggregateLogic::All => u_bools.into_iter().reduce(|acc, u| acc & u).unwrap(),
        AggregateLogic::Any => u_bools.into_iter().reduce(|acc, u| acc | u).unwrap(),
        AggregateLogic::None => !u_bools.into_iter().reduce(|acc, u| acc | u).unwrap(),
        AggregateLogic::Some(k) => {
            let bools: Result<Vec<bool>, _> = u_bools
                .iter()
                .map(|u| u.to_bool(threshold, 0.95, 0.05, 1000))
                .collect();
            let true_count = bools?.iter().filter(|&&b| b).count();
            Uncertain::<bool>::point(true_count >= *k)
        }
    };
    Ok(EffectValue::UncertainBool(final_ubool))
}
