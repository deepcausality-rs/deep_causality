/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{CausalEffect, EffectLog};
use deep_causality_core::{CausalEffectPropagationProcess, CausalityError, CausalityErrorEnum};
use deep_causality_haft::LogAddEntry;

#[test]
fn test_explain_value_only() {
    let process: CausalEffectPropagationProcess<i32, (), (), CausalityError, EffectLog> =
        CausalEffectPropagationProcess::new(
            Ok(CausalEffect::value(42)),
            (),
            None,
            EffectLog::new(),
        );

    let explanation = process.explain();

    assert!(explanation.contains("Final Value: Value(42)"));
    assert!(!explanation.contains("Error:"));
    assert!(!explanation.contains("--- Logs ---"));
}

#[test]
fn test_explain_with_error() {
    // Value and error are one channel: an errored process holds no value, so
    // explain() prints the error arm INSTEAD of a final value.
    let error = CausalityError::new(CausalityErrorEnum::InternalLogicError);
    let process: CausalEffectPropagationProcess<i32, (), (), CausalityError, EffectLog> =
        CausalEffectPropagationProcess::new(Err(error), (), None, EffectLog::new());

    let explanation = process.explain();

    assert!(!explanation.contains("Final Value:"));
    assert!(explanation.contains("Error:"));
    assert!(explanation.contains("InternalLogicError"));
    assert!(!explanation.contains("--- Logs ---"));
}

#[test]
fn test_explain_with_logs() {
    let mut logs = EffectLog::new();
    logs.add_entry("Step 1: Processing data");
    logs.add_entry("Step 2: Computing result");

    let process: CausalEffectPropagationProcess<i32, (), (), CausalityError, EffectLog> =
        CausalEffectPropagationProcess::new(Ok(CausalEffect::value(100)), (), None, logs);

    let explanation = process.explain();

    assert!(explanation.contains("Final Value: Value(100)"));
    assert!(!explanation.contains("Error:"));
    assert!(explanation.contains("--- Logs ---"));
    assert!(explanation.contains("Step 1: Processing data"));
    assert!(explanation.contains("Step 2: Computing result"));
}

#[test]
fn test_explain_with_error_and_logs() {
    // Formerly "with_all_fields": a process can no longer hold a value AND an
    // error, so the richest errored explanation is the error arm plus the logs.
    let mut logs = EffectLog::new();
    logs.add_entry("Initial computation");
    logs.add_entry("Final computation");

    let error = CausalityError::new(CausalityErrorEnum::InternalLogicError);
    let process: CausalEffectPropagationProcess<i32, (), (), CausalityError, EffectLog> =
        CausalEffectPropagationProcess::new(Err(error), (), None, logs);

    let explanation = process.explain();

    assert!(!explanation.contains("Final Value:"));
    assert!(explanation.contains("Error:"));
    assert!(explanation.contains("InternalLogicError"));
    assert!(explanation.contains("--- Logs ---"));
    assert!(explanation.contains("Initial computation"));
    assert!(explanation.contains("Final computation"));
}

#[test]
fn test_explain_with_effect_value_none() {
    let process: CausalEffectPropagationProcess<i32, (), (), CausalityError, EffectLog> =
        CausalEffectPropagationProcess::new(Ok(CausalEffect::none()), (), None, EffectLog::new());

    let explanation = process.explain();

    assert!(explanation.contains("Final Value: None"));
    assert!(!explanation.contains("Error:"));
    assert!(!explanation.contains("--- Logs ---"));
}

#[test]
fn test_explain_empty_logs_no_section() {
    let logs = EffectLog::new();

    let process: CausalEffectPropagationProcess<i32, (), (), CausalityError, EffectLog> =
        CausalEffectPropagationProcess::new(Ok(CausalEffect::value(1)), (), None, logs);

    let explanation = process.explain();

    // Empty logs should not show the logs section
    assert!(!explanation.contains("--- Logs ---"));
}
