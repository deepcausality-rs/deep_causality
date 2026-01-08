/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{CausalEffectPropagationProcess, CausalityError, CausalityErrorEnum};
use deep_causality_core::{EffectLog, EffectValue};
use deep_causality_haft::LogAddEntry;

#[test]
fn test_explain_value_only() {
    let process = CausalEffectPropagationProcess {
        value: EffectValue::Value(42),
        state: (),
        context: None::<()>,
        error: None::<CausalityError>,
        logs: EffectLog::new(),
    };

    let explanation = process.explain();

    assert!(explanation.contains("Final Value: Value(42)"));
    assert!(!explanation.contains("Error:"));
    assert!(!explanation.contains("--- Logs ---"));
}

#[test]
fn test_explain_with_error() {
    let error = CausalityError::new(CausalityErrorEnum::InternalLogicError);
    let process = CausalEffectPropagationProcess {
        value: EffectValue::Value(42),
        state: (),
        context: None::<()>,
        error: Some(error),
        logs: EffectLog::new(),
    };

    let explanation = process.explain();

    assert!(explanation.contains("Final Value: Value(42)"));
    assert!(explanation.contains("Error:"));
    assert!(explanation.contains("InternalLogicError"));
    assert!(!explanation.contains("--- Logs ---"));
}

#[test]
fn test_explain_with_logs() {
    let mut logs = EffectLog::new();
    logs.add_entry("Step 1: Processing data");
    logs.add_entry("Step 2: Computing result");

    let process = CausalEffectPropagationProcess {
        value: EffectValue::Value(100),
        state: (),
        context: None::<()>,
        error: None::<CausalityError>,
        logs,
    };

    let explanation = process.explain();

    assert!(explanation.contains("Final Value: Value(100)"));
    assert!(!explanation.contains("Error:"));
    assert!(explanation.contains("--- Logs ---"));
    assert!(explanation.contains("Step 1: Processing data"));
    assert!(explanation.contains("Step 2: Computing result"));
}

#[test]
fn test_explain_with_all_fields() {
    let mut logs = EffectLog::new();
    logs.add_entry("Initial computation");
    logs.add_entry("Final computation");

    let error = CausalityError::new(CausalityErrorEnum::InternalLogicError);
    let process = CausalEffectPropagationProcess {
        value: EffectValue::Value(999),
        state: (),
        context: None::<()>,
        error: Some(error),
        logs,
    };

    let explanation = process.explain();

    assert!(explanation.contains("Final Value: Value(999)"));
    assert!(explanation.contains("Error:"));
    assert!(explanation.contains("InternalLogicError"));
    assert!(explanation.contains("--- Logs ---"));
    assert!(explanation.contains("Initial computation"));
    assert!(explanation.contains("Final computation"));
}

#[test]
fn test_explain_with_effect_value_none() {
    let process = CausalEffectPropagationProcess {
        value: EffectValue::<i32>::None,
        state: (),
        context: None::<()>,
        error: None::<CausalityError>,
        logs: EffectLog::new(),
    };

    let explanation = process.explain();

    assert!(explanation.contains("Final Value: None"));
    assert!(!explanation.contains("Error:"));
    assert!(!explanation.contains("--- Logs ---"));
}

#[test]
fn test_explain_empty_logs_no_section() {
    let logs = EffectLog::new();

    let process = CausalEffectPropagationProcess {
        value: EffectValue::Value(1),
        state: (),
        context: None::<()>,
        error: None::<CausalityError>,
        logs,
    };

    let explanation = process.explain();

    // Empty logs should not show the logs section
    assert!(!explanation.contains("--- Logs ---"));
}
