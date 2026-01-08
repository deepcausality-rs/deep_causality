/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::Identifiable;
use deep_causality::{ActionParameterValue, BaseContext, ProposedAction};
use deep_causality_ethos::{Teloid, TeloidModal};
use std::collections::HashMap;

// Mock activation predicate functions
fn always_true_predicate(_context: &BaseContext, _action: &ProposedAction) -> bool {
    true
}

fn check_speed_predicate(_context: &BaseContext, action: &ProposedAction) -> bool {
    if let Some(ActionParameterValue::Number(speed)) = action.parameters().get("speed") {
        *speed > 50.0
    } else {
        false
    }
}

// Helper to create a dummy context (since Teloid doesn't own it, we just need a placeholder)
fn get_dummy_context() -> BaseContext {
    BaseContext::with_capacity(0, "dummy_context", 1)
}

// Helper to create a dummy proposed action
fn get_dummy_action(speed: f64) -> ProposedAction {
    let mut params = HashMap::new();
    params.insert("speed".to_string(), ActionParameterValue::Number(speed));
    let action_name = "TestSpeedAction".to_string();
    ProposedAction::new(0, action_name, params)
}

#[test]
fn test_teloid_new() {
    let teloid = Teloid::new_deterministic(
        1,
        "action.test".to_string(),
        always_true_predicate,
        TeloidModal::Obligatory,
        100,
        10,
        5,
        vec!["tag1", "tag2"],
        None,
    );

    assert_eq!(teloid.id(), 1);
    assert_eq!(teloid.action_identifier(), "action.test");
    assert_eq!(teloid.modality(), TeloidModal::Obligatory);
    assert_eq!(teloid.timestamp(), 100);
    assert_eq!(teloid.specificity(), 10);
    assert_eq!(teloid.priority(), 5);
    assert_eq!(teloid.tags(), &vec!["tag1", "tag2"]);
    assert!(teloid.metadata().is_none());
}

#[test]
fn test_teloid_new_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("author".to_string(), "test_author".to_string());

    let teloid = Teloid::new_deterministic(
        2,
        "action.meta".to_string(),
        always_true_predicate,
        TeloidModal::Impermissible,
        200,
        20,
        10,
        vec![],
        Some(metadata.clone()),
    );

    assert_eq!(teloid.id(), 2);
    assert_eq!(teloid.metadata(), &Some(metadata));
}

#[test]
fn test_teloid_getters() {
    let teloid = Teloid::new_deterministic(
        3,
        "action.get".to_string(),
        always_true_predicate,
        TeloidModal::Optional(50),
        300,
        30,
        15,
        vec!["get_tag"],
        None,
    );

    assert_eq!(teloid.id(), 3);
    assert_eq!(teloid.action_identifier(), "action.get");
    assert!(teloid.activation_predicate().unwrap()(
        &get_dummy_context(),
        &get_dummy_action(0.0)
    ),);
    assert_eq!(teloid.modality(), TeloidModal::Optional(50));
    assert_eq!(teloid.timestamp(), 300);
    assert_eq!(teloid.specificity(), 30);
    assert_eq!(teloid.priority(), 15);
    assert_eq!(teloid.tags(), &vec!["get_tag"]);
    assert!(teloid.metadata().is_none());
    assert!(teloid.uncertain_activation_predicate().is_none());
    assert!(teloid.uncertain_parameter().is_none());
}

#[test]
fn test_teloid_clone() {
    let teloid = Teloid::new_deterministic(
        4,
        "action.clone".to_string(),
        always_true_predicate,
        TeloidModal::Obligatory,
        400,
        40,
        20,
        vec!["clone_tag"],
        None,
    );
    let cloned_teloid = teloid.clone();

    assert_eq!(teloid, cloned_teloid);
    assert_eq!(teloid.id(), cloned_teloid.id());
    assert_eq!(
        teloid.action_identifier(),
        cloned_teloid.action_identifier()
    );
    assert_eq!(teloid.modality(), cloned_teloid.modality());
}

#[test]
fn test_teloid_equality() {
    let teloid1 = Teloid::new_deterministic(
        5,
        "action.eq".to_string(),
        always_true_predicate,
        TeloidModal::Obligatory,
        500,
        50,
        25,
        vec!["eq_tag"],
        None,
    );
    let teloid2 = Teloid::new_deterministic(
        5,
        "action.eq_diff".to_string(), // Different action_identifier
        check_speed_predicate,        // Different predicate
        TeloidModal::Impermissible,   // Different modality
        501,                          // Different timestamp
        51,                           // Different specificity
        26,                           // Different priority
        vec!["eq_tag_diff"],          // Different tags
        Some(HashMap::new()),         // Different metadata
    );
    let teloid3 = Teloid::new_deterministic(
        6,
        "action.eq".to_string(),
        always_true_predicate,
        TeloidModal::Obligatory,
        500,
        50,
        25,
        vec!["eq_tag"],
        None,
    );

    assert_eq!(teloid1, teloid2); // Should be equal because only ID matters for PartialEq
    assert_ne!(teloid1, teloid3); // Different ID
}

#[test]
fn test_teloid_display() {
    let teloid = Teloid::new_deterministic(
        7,
        "action.display".to_string(),
        always_true_predicate,
        TeloidModal::Optional(100),
        700,
        70,
        35,
        vec!["disp_tag"],
        None,
    );
    let expected_display = "Teloid { id: 7, action_identifier: \"action.display\", modality: Optional(100), timestamp: 700, specificity: 70, priority: 35, tags: [\"disp_tag\"] }";
    assert_eq!(format!("{}", teloid), expected_display);
}

#[test]
fn test_teloid_activation_predicate_execution() {
    let context = get_dummy_context();
    let action_fast = get_dummy_action(60.0);
    let action_slow = get_dummy_action(40.0);

    let teloid_speed_check = Teloid::new_deterministic(
        9,
        "action.speed_check".to_string(),
        check_speed_predicate,
        TeloidModal::Impermissible,
        900,
        90,
        45,
        vec!["speed_tag"],
        None,
    );

    assert!(teloid_speed_check.activation_predicate().unwrap()(
        &context,
        &action_fast
    ));
    assert!(!teloid_speed_check.activation_predicate().unwrap()(
        &context,
        &action_slow
    ));
}

#[test]
fn test_teloid_identifiable_trait() {
    let teloid = Teloid::new_deterministic(
        42,
        "action.identifiable".to_string(),
        always_true_predicate,
        TeloidModal::Obligatory,
        1,
        1,
        1,
        vec!["id_tag"],
        None,
    );

    // Explicitly test the Identifiable trait implementation
    let identifiable: &dyn Identifiable = &teloid;
    assert_eq!(identifiable.id(), 42);
    // Also check the direct method call for consistency
    assert_eq!(teloid.id(), 42);
}
