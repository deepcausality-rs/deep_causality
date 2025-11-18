/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{CausalMonad, CausalPropagatingEffect, CausalityError, EffectValue};
use deep_causality_haft::MonadEffect3;

use deep_causality::utils_test::test_utils_monad::*;

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
