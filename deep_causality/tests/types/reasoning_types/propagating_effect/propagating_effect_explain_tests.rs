/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{CausalEffectLog, CausalityError, PropagatingEffect};

#[test]
fn test_explain_value_only() {
    let effect = PropagatingEffect::from_boolean(true);
    let explanation = effect.explain();
    dbg!(&explanation);

    assert!(explanation.starts_with("Final Value: Boolean(true)"));
    assert!(!explanation.contains("Error:"));
    assert!(!explanation.contains("--- Logs ---"));
}

#[test]
fn test_explain_with_error() {
    let error = CausalityError::new("Test error".to_string());
    let effect = PropagatingEffect::from_error(error);
    let explanation = effect.explain();
    dbg!(&explanation);

    assert!(explanation.starts_with("Final Value: None"));
    assert!(explanation.contains("Error: CausalityError(\"Test error\")"));
    assert!(!explanation.contains("--- Logs ---"));
}

#[test]
fn test_explain_with_log() {
    let mut effect = PropagatingEffect::from_boolean(false);
    let mut log = CausalEffectLog::new();
    log.add_entry("Test log entry");
    effect.logs = log;
    let explanation = effect.explain();
    dbg!(&explanation);

    assert!(explanation.starts_with("Final Value: Boolean(false)"));
    assert!(!explanation.contains("Error:"));
    assert!(explanation.contains("--- Logs ---"));
    assert!(explanation.contains("Test log entry"));
}

#[test]
fn test_explain_with_error_and_log() {
    let error = CausalityError::new("Another test error".to_string());
    let mut effect = PropagatingEffect::from_error(error);
    let mut log = CausalEffectLog::new();
    log.add_entry("Another test log entry");
    effect.logs = log;
    let explanation = effect.explain();
    dbg!(&explanation);

    assert!(explanation.starts_with("Final Value: None"));
    assert!(explanation.contains("Error: CausalityError(\"Another test error\")"));
    assert!(explanation.contains("--- Logs ---"));
    assert!(explanation.contains("Another test log entry"));
}
