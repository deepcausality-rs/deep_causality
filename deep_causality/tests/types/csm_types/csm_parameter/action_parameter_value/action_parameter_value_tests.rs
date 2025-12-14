/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::ActionParameterValue;

// ============================================================================
// Enum Variant Tests
// ============================================================================

#[test]
fn test_action_parameter_value_string() {
    let value = ActionParameterValue::String("hello".to_string());
    match &value {
        ActionParameterValue::String(s) => assert_eq!(s, "hello"),
        _ => panic!("Expected String variant"),
    }
}

#[test]
fn test_action_parameter_value_number() {
    let value = ActionParameterValue::Number(1.23456);
    match value {
        ActionParameterValue::Number(n) => assert!((n - 1.23456).abs() < 1e-10),
        _ => panic!("Expected Number variant"),
    }
}

#[test]
fn test_action_parameter_value_integer() {
    let value = ActionParameterValue::Integer(42);
    match value {
        ActionParameterValue::Integer(i) => assert_eq!(i, 42),
        _ => panic!("Expected Integer variant"),
    }
}

#[test]
fn test_action_parameter_value_boolean_true() {
    let value = ActionParameterValue::Boolean(true);
    match value {
        ActionParameterValue::Boolean(b) => assert!(b),
        _ => panic!("Expected Boolean variant"),
    }
}

#[test]
fn test_action_parameter_value_boolean_false() {
    let value = ActionParameterValue::Boolean(false);
    match value {
        ActionParameterValue::Boolean(b) => assert!(!b),
        _ => panic!("Expected Boolean variant"),
    }
}

#[test]
fn test_action_parameter_value_contextual_link() {
    let context_id = 100u64;
    let contextoid_id = 200u64;
    let value = ActionParameterValue::ContextualLink(context_id, contextoid_id);
    match value {
        ActionParameterValue::ContextualLink(cid, ctxoid) => {
            assert_eq!(cid, 100);
            assert_eq!(ctxoid, 200);
        }
        _ => panic!("Expected ContextualLink variant"),
    }
}

// ============================================================================
// Display Trait Tests
// ============================================================================

#[test]
fn test_display_string() {
    let value = ActionParameterValue::String("test".to_string());
    let display = format!("{}", value);
    assert_eq!(display, "ActionParameterValue::String: test");
}

#[test]
fn test_display_number() {
    let value = ActionParameterValue::Number(1.5);
    let display = format!("{}", value);
    assert_eq!(display, "ActionParameterValue::Number: 1.50");
}

#[test]
fn test_display_integer() {
    let value = ActionParameterValue::Integer(-100);
    let display = format!("{}", value);
    assert_eq!(display, "ActionParameterValue::Integer: -100");
}

#[test]
fn test_display_boolean() {
    let value = ActionParameterValue::Boolean(true);
    let display = format!("{}", value);
    assert_eq!(display, "ActionParameterValue::Boolean: true");
}

#[test]
fn test_display_contextual_link() {
    let value = ActionParameterValue::ContextualLink(1, 2);
    let display = format!("{}", value);
    assert_eq!(display, "ActionParameterValue::ContextualLink(1, 2)");
}

// ============================================================================
// Clone and PartialEq Trait Tests
// ============================================================================

#[test]
fn test_clone() {
    let original = ActionParameterValue::String("clone me".to_string());
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_partial_eq_equal() {
    let a = ActionParameterValue::Number(42.0);
    let b = ActionParameterValue::Number(42.0);
    assert_eq!(a, b);
}

#[test]
fn test_partial_eq_not_equal_same_variant() {
    let a = ActionParameterValue::Integer(1);
    let b = ActionParameterValue::Integer(2);
    assert_ne!(a, b);
}

#[test]
fn test_partial_eq_not_equal_different_variant() {
    let a = ActionParameterValue::Integer(42);
    let b = ActionParameterValue::Number(42.0);
    assert_ne!(a, b);
}

// ============================================================================
// Debug Trait Tests
// ============================================================================

#[test]
fn test_debug() {
    let value = ActionParameterValue::Boolean(true);
    let debug = format!("{:?}", value);
    assert!(debug.contains("Boolean"));
    assert!(debug.contains("true"));
}
