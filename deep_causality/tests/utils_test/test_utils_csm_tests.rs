/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils_csm::{
    get_test_action_with_tracker, get_test_causaloid, get_test_error_action,
    get_test_error_causaloid, get_test_probabilistic_causaloid,
};
use deep_causality::{EffectValue, MonadicCausable, PropagatingEffect};

#[test]
fn test_get_test_error_action() {
    let action = get_test_error_action();
    assert_eq!(action.version(), 1);
    assert_eq!(action.description(), "Test action");
    assert!(action.fire().is_err());
    assert_eq!(action.fire().unwrap_err().to_string(), "ActionError: Error");
}

#[test]
fn test_get_test_probabilistic_causaloid() {
    let causaloid = get_test_probabilistic_causaloid();
    assert_eq!(causaloid.id(), 99);
    assert_eq!(causaloid.description(), "Probabilistic Causaloid");

    // Evaluate to exercise the causal function body.
    let effect = PropagatingEffect::from_value(1.0f64);
    let res = causaloid.evaluate(&effect);
    assert!(res.error.is_none());
    assert_eq!(res.value, EffectValue::Value(0.5));
}

#[test]
fn test_get_test_error_causaloid() {
    let causaloid = get_test_error_causaloid();
    assert_eq!(causaloid.id(), 78);
    assert_eq!(causaloid.description(), "Error Causaloid");
}

#[test]
fn test_get_test_causaloid_with_context() {
    let causaloid = get_test_causaloid(true);
    assert_eq!(causaloid.id(), 1);
    assert_eq!(causaloid.description(), "Inverts any input");
    assert!(causaloid.context().is_some());
}

#[test]
fn test_get_test_causaloid_without_context() {
    let causaloid = get_test_causaloid(false);
    assert_eq!(causaloid.id(), 1);
    assert_eq!(causaloid.description(), "Test Causaloid");
    assert!(causaloid.context().is_none());

    // Evaluate to exercise the context-less causal function body, which always
    // returns `true` and logs an entry.
    let effect = PropagatingEffect::from_value(false);
    let res = causaloid.evaluate(&effect);
    assert!(res.error.is_none());
    assert_eq!(res.value, EffectValue::Value(true));
}

#[test]
fn test_get_test_action_with_tracker() {
    let action = get_test_action_with_tracker();
    assert_eq!(action.version(), 1);
    assert_eq!(action.description(), "Tracked Action");
    // The action function itself contains the tracker logic, which is not directly accessible here.
    // We can only test that firing it doesn't return an error.
    assert!(action.fire().is_ok());
}
