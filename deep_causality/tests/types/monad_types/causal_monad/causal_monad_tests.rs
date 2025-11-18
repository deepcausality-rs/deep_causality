/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    CausalEffectLog, CausalMonad, CausalPropagatingEffect, CausalityError, EffectValue,
    NumericalValue,
};
use deep_causality_haft::MonadEffect3;

// Helper function from the example to simulate a causal step
fn smoking_logic(
    nicotine_obs: EffectValue,
) -> CausalPropagatingEffect<EffectValue, CausalityError, CausalEffectLog> {
    let mut log = CausalEffectLog::new();
    let nicotine_level = nicotine_obs.as_numerical().unwrap_or(&0.0);
    let threshold: NumericalValue = 0.6;
    let high_nicotine = nicotine_level > &threshold;
    log.add_entry(&format!(
        "Nicotine level {} is higher than threshold {}: {}",
        nicotine_obs, threshold, high_nicotine
    ));
    CausalPropagatingEffect::from_effect_value_with_log(EffectValue::Boolean(high_nicotine), log)
}

// Another helper function from the example
fn tar_logic(
    is_smoking: EffectValue,
) -> CausalPropagatingEffect<EffectValue, CausalityError, CausalEffectLog> {
    let mut log = CausalEffectLog::new();
    let has_tar = is_smoking.as_bool().unwrap_or(false);
    log.add_entry(&format!("Has tar in lung {}", has_tar));
    CausalPropagatingEffect::from_effect_value_with_log(EffectValue::Boolean(has_tar), log)
}

// Helper function that can introduce an error
fn error_logic(
    input: EffectValue,
) -> CausalPropagatingEffect<EffectValue, CausalityError, CausalEffectLog> {
    let mut log = CausalEffectLog::new();
    log.add_entry("Error logic applied");
    if input.as_bool().unwrap_or(false) {
        CausalPropagatingEffect {
            value: EffectValue::None,
            error: Some(CausalityError::new("Simulated error".to_string())),
            logs: log, // Preserve the logs
        }
    } else {
        CausalPropagatingEffect::from_effect_value_with_log(input, log)
    }
}

#[test]
fn test_pure_boolean() {
    let effect = CausalMonad::pure(EffectValue::Boolean(true));
    assert!(matches!(effect.value, EffectValue::Boolean(true)));
    assert!(!effect.is_error());
    assert!(effect.logs.is_empty());
}

#[test]
fn test_pure_numerical() {
    let effect = CausalMonad::pure(EffectValue::Numerical(123.45));
    assert!(matches!(effect.value, EffectValue::Numerical(val) if val == 123.45));
    assert!(!effect.is_error());
    assert!(effect.logs.is_empty());
}

#[test]
fn test_bind_success_and_log_aggregation() {
    let initial_effect = CausalMonad::pure(EffectValue::Numerical(0.7));

    let final_effect = initial_effect.bind(smoking_logic).bind(tar_logic);

    assert!(matches!(final_effect.value, EffectValue::Boolean(true)));
    assert!(!final_effect.is_error());
    assert_eq!(final_effect.logs.len(), 2);
    assert!(final_effect.logs.to_string().contains("Nicotine level"));
    assert!(final_effect.logs.to_string().contains("Has tar in lung"));
}

#[test]
fn test_bind_error_propagation() {
    let initial_effect = CausalMonad::pure(EffectValue::Boolean(true)); // This will trigger the error_logic

    let final_effect = initial_effect
        .bind(error_logic) // This step introduces an error
        .bind(tar_logic); // This step should be short-circuited

    assert!(matches!(final_effect.value, EffectValue::None)); // Default value for U is EffectValue::None
    assert!(final_effect.is_error());
    assert_eq!(final_effect.error.unwrap().0, "Simulated error");
    assert_eq!(final_effect.logs.len(), 1); // Only logs from error_logic, not tar_logic
    assert!(
        final_effect
            .logs
            .to_string()
            .contains("Error logic applied")
    );
}

#[test]
fn test_intervene_value_replacement() {
    let initial_effect = CausalMonad::pure(EffectValue::Numerical(0.9)); // High nicotine
    let intervened_effect = initial_effect.intervene(EffectValue::Numerical(0.1)); // Force low nicotine

    assert!(matches!(intervened_effect.value, EffectValue::Numerical(val) if val == 0.1));
    assert!(!intervened_effect.is_error());
    assert_eq!(intervened_effect.logs.len(), 1);
    assert!(
        intervened_effect
            .logs
            .to_string()
            .contains("Intervention: Value replaced with Numerical(0.1)")
    );
}

#[test]
fn test_intervene_preserves_error() {
    let initial_effect =
        CausalPropagatingEffect::from_error(CausalityError::new("Original error".to_string()));
    let intervened_effect = initial_effect.intervene(EffectValue::Boolean(true));

    assert!(matches!(
        intervened_effect.value,
        EffectValue::Boolean(true)
    ));
    assert!(intervened_effect.is_error());
    assert_eq!(intervened_effect.error.unwrap().0, "Original error");
    assert_eq!(intervened_effect.logs.len(), 1);
    assert!(
        intervened_effect
            .logs
            .to_string()
            .contains("Intervention: Value replaced with Boolean(true)")
    );
}

#[test]
fn test_intervene_in_chain() {
    let initial_effect = CausalMonad::pure(EffectValue::Numerical(0.9)); // High nicotine

    let final_effect = initial_effect
        .bind(smoking_logic) // smoking_logic would return true
        .intervene(EffectValue::Boolean(false)) // Force smoking to false
        .bind(tar_logic); // tar_logic should now receive false

    assert!(matches!(final_effect.value, EffectValue::Boolean(false)));
    assert!(!final_effect.is_error());
    assert_eq!(final_effect.logs.len(), 3);
    assert!(final_effect.logs.to_string().contains("Nicotine level"));
    assert!(
        final_effect
            .logs
            .to_string()
            .contains("Intervention: Value replaced with Boolean(false)")
    );
    assert!(
        final_effect
            .logs
            .to_string()
            .contains("Has tar in lung false")
    );
}

#[test]
fn test_bind_with_default_value_on_error() {
    let initial_effect = CausalMonad::pure(EffectValue::Boolean(true)); // Triggers error
    let final_effect = initial_effect
        .bind(error_logic)
        .bind(|_val: EffectValue| CausalMonad::pure(EffectValue::Numerical(100.0))); // This bind should receive default

    assert!(matches!(final_effect.value, EffectValue::None)); // Default for U is EffectValue::None
    assert!(final_effect.is_error());
    assert_eq!(final_effect.error.unwrap().0, "Simulated error");
    assert_eq!(final_effect.logs.len(), 1);
    assert!(
        final_effect
            .logs
            .to_string()
            .contains("Error logic applied")
    );
}
