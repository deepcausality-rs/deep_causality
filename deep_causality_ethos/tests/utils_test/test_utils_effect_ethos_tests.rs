/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::Identifiable;
use deep_causality::{ActionParameterValue, ProposedAction};
use deep_causality_ethos::utils_test::test_utils_effect_ethos::{
    always_false_predicate, always_true_predicate, check_speed_predicate, get_dummy_action,
    get_dummy_context,
};

#[test]
fn test_always_true_predicate() {
    let context = get_dummy_context();
    let action = get_dummy_action("test", 0.0);
    assert!(always_true_predicate(&context, &action));
}

#[test]
fn test_always_false_predicate() {
    let context = get_dummy_context();
    let action = get_dummy_action("test", 0.0);
    assert!(!always_false_predicate(&context, &action));
}

#[test]
fn test_check_speed_predicate_true() {
    let context = get_dummy_context();
    let action = get_dummy_action("fast_action", 60.0);
    assert!(check_speed_predicate(&context, &action));
}

#[test]
fn test_check_speed_predicate_false_below_threshold() {
    let context = get_dummy_context();
    let action = get_dummy_action("slow_action", 40.0);
    assert!(!check_speed_predicate(&context, &action));
}

#[test]
fn test_check_speed_predicate_false_no_speed_param() {
    let context = get_dummy_context();
    let action = ProposedAction::new(0, "no_speed".to_string(), std::collections::HashMap::new());
    assert!(!check_speed_predicate(&context, &action));
}

#[test]
fn test_check_speed_predicate_false_wrong_param_type() {
    let context = get_dummy_context();
    let mut params = std::collections::HashMap::new();
    params.insert("speed".to_string(), ActionParameterValue::Boolean(true));
    let action = ProposedAction::new(0, "wrong_type".to_string(), params);
    assert!(!check_speed_predicate(&context, &action));
}

#[test]
fn test_get_dummy_context() {
    let context = get_dummy_context();
    assert_eq!(context.id(), 0);
    assert_eq!(context.name(), "dummy_context");
}

#[test]
fn test_get_dummy_action() {
    let action = get_dummy_action("test_action", 75.0);
    assert_eq!(action.id(), 0);
    assert_eq!(
        action.parameters().get("speed"),
        Some(&ActionParameterValue::Number(75.0))
    );
}
