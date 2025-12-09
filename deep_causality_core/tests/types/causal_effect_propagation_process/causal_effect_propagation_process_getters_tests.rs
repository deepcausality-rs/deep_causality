/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{CausalEffectPropagationProcess, EffectValue};

#[test]
fn test_causal_effect_propagation_process_getters() {
    // Setup test data
    let value = EffectValue::Value(42);
    let state = "TestState".to_string();
    let context = Some("TestContext".to_string());
    let error: Option<String> = Some("TestError".to_string());
    let logs = vec!["Log1".to_string(), "Log2".to_string()];

    // Create process
    let process = CausalEffectPropagationProcess {
        value: value.clone(),
        state: state.clone(),
        context: context.clone(),
        error: error.clone(),
        logs: logs.clone(),
    };

    // Test Getters
    assert_eq!(process.value(), &value);
    assert_eq!(process.state(), &state);
    assert_eq!(process.context(), &context);
    assert_eq!(process.error(), &error);
    assert_eq!(process.logs(), &logs);
}

#[test]
fn test_causal_effect_propagation_process_getters_defaults() {
    // Setup test data with "None" options/empty logs
    let value = EffectValue::Value(100);
    let state = 0;
    let context: Option<i32> = None;
    let error: Option<String> = None;
    let logs: Vec<String> = Vec::new();

    let process = CausalEffectPropagationProcess {
        value: value.clone(),
        state,
        context,
        error,
        logs: logs.clone(),
    };

    assert_eq!(process.value(), &value);
    assert_eq!(*process.state(), 0);
    assert_eq!(process.context(), &None);
    assert_eq!(process.error(), &None);
    assert!(process.logs().is_empty());
}
