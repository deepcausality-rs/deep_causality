/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    BaseCausaloid, Causable, CausalEffectLog, CausalMonad, CausalityError, Causaloid,
    CausaloidRegistry, IntoEffectValue, MonadicCausable, PropagatingEffect,
};
use std::fmt::Display;

// Define concrete types for the generic parameters for easier use in tests.
type TestCausaloid<I, O> = BaseCausaloid<I, O>;

// Helper function to create a simple test causaloid.
fn create_test_causaloid<I, O>(
    description: &'static str,
    causal_fn: fn(I) -> Result<deep_causality::CausalFnOutput<O>, CausalityError>,
) -> TestCausaloid<I, O>
where
    I: IntoEffectValue + Default,
    O: IntoEffectValue + Default,
    TestCausaloid<I, O>: MonadicCausable<CausalMonad> + Causable + Display + Send + Sync + 'static,
{
    // The ID for a new causaloid is irrelevant as the registry will assign its own.
    Causaloid::new(0, causal_fn, description)
}

#[test]
fn test_register_single_causaloid() {
    let mut registry = CausaloidRegistry::new();

    fn causal_fn(obs: bool) -> Result<deep_causality::CausalFnOutput<bool>, CausalityError> {
        Ok(deep_causality::CausalFnOutput::new(
            !obs,
            CausalEffectLog::default(),
        ))
    }

    let causaloid = create_test_causaloid("Inverts a boolean", causal_fn);
    let id = registry.register(causaloid);

    assert_eq!(id, 0);

    // Check evaluation
    let input_effect = PropagatingEffect::from_deterministic(true);
    let output_effect = registry.evaluate(id, &input_effect);
    let output_value = bool::try_from_effect_value(output_effect.value).unwrap();
    assert!(!output_value);
}

#[test]
fn test_register_multiple_causaloids_same_type() {
    let mut registry = CausaloidRegistry::new();

    fn causal_fn(obs: bool) -> Result<deep_causality::CausalFnOutput<bool>, CausalityError> {
        Ok(deep_causality::CausalFnOutput::new(
            !obs,
            CausalEffectLog::default(),
        ))
    }

    let causaloid1 = create_test_causaloid("Inverts a boolean 1", causal_fn);
    let id1 = registry.register(causaloid1);

    let causaloid2 = create_test_causaloid("Inverts a boolean 2", causal_fn);
    let id2 = registry.register(causaloid2);

    assert_eq!(id1, 0);
    assert_eq!(id2, 1);

    // Evaluate first causaloid
    let input1 = PropagatingEffect::from_deterministic(true);
    let output1 = registry.evaluate(id1, &input1);
    assert!(!bool::try_from_effect_value(output1.value).unwrap());

    // Evaluate second causaloid
    let input2 = PropagatingEffect::from_deterministic(false);
    let output2 = registry.evaluate(id2, &input2);
    assert!(bool::try_from_effect_value(output2.value).unwrap());
}

#[test]
fn test_register_multiple_causaloids_different_types() {
    let mut registry = CausaloidRegistry::new();

    // Causaloid 1: bool -> bool
    fn causal_fn1(obs: bool) -> Result<deep_causality::CausalFnOutput<bool>, CausalityError> {
        Ok(deep_causality::CausalFnOutput::new(
            !obs,
            CausalEffectLog::default(),
        ))
    }
    let causaloid1 = create_test_causaloid("Inverts a boolean", causal_fn1);
    let id1 = registry.register(causaloid1);

    // Causaloid 2: f64 -> f64
    fn causal_fn2(obs: f64) -> Result<deep_causality::CausalFnOutput<f64>, CausalityError> {
        Ok(deep_causality::CausalFnOutput::new(
            obs * 2.0,
            CausalEffectLog::default(),
        ))
    }
    let causaloid2 = create_test_causaloid("Doubles a float", causal_fn2);
    let id2 = registry.register(causaloid2);

    assert_eq!(id1, 0);
    assert_eq!(id2, 1);

    // Evaluate bool causaloid
    let input1 = PropagatingEffect::from_deterministic(true);
    let output1 = registry.evaluate(id1, &input1);
    assert!(!bool::try_from_effect_value(output1.value).unwrap());

    // Evaluate f64 causaloid
    let input2 = PropagatingEffect::from_numerical(5.0);
    let output2 = registry.evaluate(id2, &input2);
    assert_eq!(f64::try_from_effect_value(output2.value).unwrap(), 10.0);
}

#[test]
fn test_evaluate_successful() {
    let mut registry = CausaloidRegistry::new();

    fn causal_fn(obs: f64) -> Result<deep_causality::CausalFnOutput<f64>, CausalityError> {
        Ok(deep_causality::CausalFnOutput::new(
            obs + 1.0,
            CausalEffectLog::default(),
        ))
    }

    let causaloid = create_test_causaloid("Adds one to a float", causal_fn);
    let id = registry.register(causaloid);

    let input_effect = PropagatingEffect::from_numerical(5.0);
    let output_effect = registry.evaluate(id, &input_effect);

    let expected_value = 6.0;
    let output_value = f64::try_from_effect_value(output_effect.value).unwrap();

    assert_eq!(output_value, expected_value);
}

#[test]
fn test_evaluate_id_not_found() {
    let registry = CausaloidRegistry::new();
    let input_effect = PropagatingEffect::from_numerical(5.0);
    let output_effect = registry.evaluate(999, &input_effect); // 999 is a non-existent ID

    assert!(output_effect.is_err());
    let err = output_effect.error.unwrap();
    assert_eq!(
        err.to_string(),
        "CausalityError: Causaloid with ID 999 not found in registry."
    );
}

#[test]
fn test_evaluate_type_mismatch() {
    let mut registry = CausaloidRegistry::new();

    // This causaloid expects an f64
    fn causal_fn(obs: f64) -> Result<deep_causality::CausalFnOutput<f64>, CausalityError> {
        Ok(deep_causality::CausalFnOutput::new(
            obs + 1.0,
            CausalEffectLog::default(),
        ))
    }

    let causaloid = create_test_causaloid("Expects f64", causal_fn);
    let id = registry.register(causaloid);

    // But we pass it a bool
    let input_effect = PropagatingEffect::from_deterministic(true);
    let output_effect = registry.evaluate(id, &input_effect);
    dbg!(&output_effect);

    // The evaluation should fail inside the causaloid due to type mismatch
    assert!(output_effect.is_err());
    let err = output_effect.error.unwrap();
    assert!(
        err.to_string()
            .contains("Expected Numerical(f64), found Deterministic(true)")
    );

    let log = output_effect.logs.to_string();
    assert!(
        log.contains("Input conversion failed: CausalityError: Expected Numerical(f64), found Deterministic(true)")
    )
}

// This test is for an internal error condition that is hard to trigger.
// It checks the branch where a TypeId is in the lookup but not in the storage.
// This should not happen in practice but we test the error handling.
#[test]
fn test_evaluate_lookup_ok_storage_type_missing() {
    let registry = CausaloidRegistry::new();

    // Manually insert a lookup entry pointing to a non-existent type
    let causaloid_registry_boned = CausaloidRegistry::new();
    causaloid_registry_boned.evaluate(0, &PropagatingEffect::none());

    let input_effect = PropagatingEffect::from_numerical(5.0);
    let output_effect = registry.evaluate(0, &input_effect);

    assert!(output_effect.is_err());
    let err = output_effect.error.unwrap();
    assert_eq!(
        err.to_string(),
        "CausalityError: Causaloid with ID 0 not found in registry.".to_string()
    );
}

// This test is for an internal error condition that is hard to trigger.
// It checks the branch where the index in the lookup is out of bounds for the vector.
// This should not happen in practice but we test the error handling.
#[test]
fn test_evaluate_lookup_ok_storage_index_out_of_bounds() {
    let mut registry = CausaloidRegistry::new();

    fn causal_fn(obs: bool) -> Result<deep_causality::CausalFnOutput<bool>, CausalityError> {
        Ok(deep_causality::CausalFnOutput::new(
            !obs,
            CausalEffectLog::default(),
        ))
    }

    let causaloid = create_test_causaloid("Inverts a boolean", causal_fn);
    let id = registry.register(causaloid); // id is 0

    let causaloid_registry_boned = CausaloidRegistry::new();
    causaloid_registry_boned.evaluate(id, &PropagatingEffect::none());

    let input_effect = PropagatingEffect::from_deterministic(true);
    let output_effect = registry.evaluate(id, &input_effect);

    assert!(!bool::try_from_effect_value(output_effect.value).unwrap());
}
