/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{ActionParameterValue, Identifiable, ProposedAction};
use std::collections::HashMap;

#[test]
fn test_proposed_action_new() {
    let mut params = HashMap::new();
    params.insert("speed".to_string(), ActionParameterValue::Number(60.0));
    params.insert(
        "destination".to_string(),
        ActionParameterValue::String("home".to_string()),
    );

    let action = ProposedAction::new(1, "drive".to_string(), params.clone());

    assert_eq!(action.action_id(), 1);
    assert_eq!(action.action_name(), "drive");
    assert_eq!(action.parameters(), &params);
}

#[test]
fn test_proposed_action_getters() {
    let mut params = HashMap::new();
    params.insert("temp".to_string(), ActionParameterValue::Integer(25));

    let action = ProposedAction::new(2, "monitor_env".to_string(), params.clone());

    assert_eq!(action.action_id(), 2);
    assert_eq!(action.action_name(), "monitor_env");
    assert_eq!(action.parameters(), &params);

    // Test empty parameters
    let empty_action = ProposedAction::new(3, "idle".to_string(), HashMap::new());
    assert!(empty_action.parameters().is_empty());
}

#[test]
fn test_proposed_action_clone() {
    let mut params = HashMap::new();
    params.insert("x".to_string(), ActionParameterValue::Number(1.0));
    let original_action = ProposedAction::new(4, "move".to_string(), params);

    let cloned_action = original_action.clone();

    assert_eq!(original_action, cloned_action);
    assert_eq!(original_action.action_id(), cloned_action.action_id());
    assert_eq!(original_action.action_name(), cloned_action.action_name());
    assert_eq!(original_action.parameters(), cloned_action.parameters());
}

#[test]
fn test_proposed_action_equality() {
    let mut params1 = HashMap::new();
    params1.insert("key".to_string(), ActionParameterValue::Boolean(true));
    let action1 = ProposedAction::new(5, "toggle".to_string(), params1.clone());

    let action2 = ProposedAction::new(5, "toggle".to_string(), params1.clone());
    assert_eq!(action1, action2);

    let action3 = ProposedAction::new(6, "toggle".to_string(), params1.clone()); // Different ID
    assert_ne!(action1, action3);

    let action4 = ProposedAction::new(5, "switch".to_string(), params1.clone()); // Different name
    assert_ne!(action1, action4);

    let mut params2 = HashMap::new();
    params2.insert("key".to_string(), ActionParameterValue::Boolean(false));
    let action5 = ProposedAction::new(5, "toggle".to_string(), params2); // Different parameters
    assert_ne!(action1, action5);
}

#[test]
fn test_proposed_action_display() {
    let mut params = HashMap::new();
    params.insert(
        "mode".to_string(),
        ActionParameterValue::String("auto".to_string()),
    );
    let action = ProposedAction::new(7, "set_mode".to_string(), params);

    let expected_display = format!(
        "ProposedAction {{ action_id: 7, action_name: set_mode, parameters: {{\"mode\": String(\"auto\")}} }}"
    );
    assert_eq!(format!("{}", action), expected_display);
}

#[test]
fn test_proposed_action_debug() {
    let mut params = HashMap::new();
    params.insert("value".to_string(), ActionParameterValue::Number(99.9));
    let action = ProposedAction::new(8, "read_sensor".to_string(), params);

    let expected_debug = format!(
        "ProposedAction {{ action_id: 8, action_name: \"read_sensor\", parameters: {{\"value\": Number(99.9)}} }}"
    );
    assert_eq!(format!("{:?}", action), expected_debug);
}

#[test]
fn test_proposed_action_identifiable() {
    let action = ProposedAction::new(9, "test_id".to_string(), HashMap::new());
    assert_eq!(action.id(), 9);
}
