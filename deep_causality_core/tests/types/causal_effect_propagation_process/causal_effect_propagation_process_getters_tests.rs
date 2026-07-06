/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{CausalEffectPropagationProcess, EffectValue};

#[test]
fn test_causal_effect_propagation_process_getters() {
    // Setup test data
    let value = EffectValue::Value(42);
    let state = "TestState".to_string();
    let context = Some("TestContext".to_string());
    let logs = vec!["Log1".to_string(), "Log2".to_string()];

    // Create process (Ok carrier: value and error are one channel)
    let process: CausalEffectPropagationProcess<i32, String, String, String, Vec<String>> =
        CausalEffectPropagationProcess::new(
            Ok(value.clone()),
            state.clone(),
            context.clone(),
            logs.clone(),
        );

    // Test Getters
    assert_eq!(process.outcome(), &Ok(value.clone()));
    assert_eq!(process.effect(), Some(&value));
    assert_eq!(process.state(), &state);
    assert_eq!(process.context(), &context);
    assert_eq!(process.error(), None);
    assert_eq!(process.logs(), &logs);
    assert!(process.is_ok());
    assert!(!process.is_err());
}

#[test]
fn test_causal_effect_propagation_process_getters_error() {
    // Setup test data for an errored carrier: value() lends nothing, error() lends the error.
    let error = "TestError".to_string();
    let state = "TestState".to_string();
    let logs = vec!["Log1".to_string(), "Log2".to_string()];

    let process: CausalEffectPropagationProcess<i32, String, String, String, Vec<String>> =
        CausalEffectPropagationProcess::new(Err(error.clone()), state.clone(), None, logs.clone());

    // Test Getters
    assert_eq!(process.outcome(), &Err(error.clone()));
    assert_eq!(process.value(), None);
    assert_eq!(process.state(), &state);
    assert_eq!(process.context(), &None);
    assert_eq!(process.error(), Some(&error));
    assert_eq!(process.logs(), &logs);
    assert!(process.is_err());
    assert!(!process.is_ok());
}

#[test]
fn test_causal_effect_propagation_process_getters_defaults() {
    // Setup test data with "None" options/empty logs
    let value = EffectValue::Value(100);
    let state = 0;
    let context: Option<i32> = None;
    let logs: Vec<String> = Vec::new();

    let process: CausalEffectPropagationProcess<i32, i32, i32, String, Vec<String>> =
        CausalEffectPropagationProcess::new(Ok(value.clone()), state, context, logs.clone());

    assert_eq!(process.effect(), Some(&value));
    assert_eq!(*process.state(), 0);
    assert_eq!(process.context(), &None);
    assert!(process.error().is_none());
    assert!(process.logs().is_empty());
}
