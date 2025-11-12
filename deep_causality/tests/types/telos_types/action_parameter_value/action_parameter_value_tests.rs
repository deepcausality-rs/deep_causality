/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::ActionParameterValue;

#[test]
fn test_action_parameter_value_string() {
    let val = ActionParameterValue::String("test_string".to_string());
    assert_eq!(val, ActionParameterValue::String("test_string".to_string()));
    assert_ne!(
        val,
        ActionParameterValue::String("other_string".to_string())
    );
    assert_eq!(
        format!("{}", val),
        "ActionParameterValue::String: test_string"
    );
    assert_eq!(format!("{:?}", val), "String(\"test_string\")");
}

#[test]
fn test_action_parameter_value_number() {
    let val = ActionParameterValue::Number(123.45);
    assert_eq!(val, ActionParameterValue::Number(123.45));
    assert_ne!(val, ActionParameterValue::Number(543.21));
    assert_eq!(format!("{}", val), "ActionParameterValue::Number: 123.45");
    assert_eq!(format!("{:?}", val), "Number(123.45)");

    // Corner cases for f64 equality
    let nan_val = ActionParameterValue::Number(f64::NAN);
    assert_ne!(nan_val, nan_val); // NaN is not equal to itself
    assert_ne!(nan_val, ActionParameterValue::Number(1.0));

    let zero_pos = ActionParameterValue::Number(0.0);
    let zero_neg = ActionParameterValue::Number(-0.0);
    assert_eq!(zero_pos, zero_neg); // 0.0 and -0.0 are equal
}

#[test]
fn test_action_parameter_value_integer() {
    let val = ActionParameterValue::Integer(123);
    assert_eq!(val, ActionParameterValue::Integer(123));
    assert_ne!(val, ActionParameterValue::Integer(321));
    assert_eq!(format!("{}", val), "ActionParameterValue::Integer: 123");
    assert_eq!(format!("{:?}", val), "Integer(123)");
}

#[test]
fn test_action_parameter_value_boolean() {
    let val = ActionParameterValue::Boolean(true);
    assert_eq!(val, ActionParameterValue::Boolean(true));
    assert_ne!(val, ActionParameterValue::Boolean(false));
    assert_eq!(format!("{}", val), "ActionParameterValue::Boolean: true");
    assert_eq!(format!("{:?}", val), "Boolean(true)");
}

#[test]
fn test_action_parameter_value_clone() {
    let val_str = ActionParameterValue::String("clone_me".to_string());
    let cloned_str = val_str.clone();
    assert_eq!(val_str, cloned_str);

    let val_num = ActionParameterValue::Number(987.65);
    let cloned_num = val_num.clone();
    assert_eq!(val_num, cloned_num);

    let val_int = ActionParameterValue::Integer(987);
    let cloned_int = val_int.clone();
    assert_eq!(val_int, cloned_int);

    let val_bool = ActionParameterValue::Boolean(true);
    let cloned_bool = val_bool.clone();
    assert_eq!(val_bool, cloned_bool);
}

#[test]
fn test_action_parameter_value_inequality_across_variants() {
    let val_str = ActionParameterValue::String("test".to_string());
    let val_num = ActionParameterValue::Number(1.0);
    let val_int = ActionParameterValue::Integer(1);
    let val_bool = ActionParameterValue::Boolean(true);

    assert_ne!(val_str, val_num);
    assert_ne!(val_str, val_int);
    assert_ne!(val_str, val_bool);

    assert_ne!(val_num, val_int);
    assert_ne!(val_num, val_bool);

    assert_ne!(val_int, val_bool);
}

#[test]
fn test_display() {
    let val_str = ActionParameterValue::String("test".to_string());
    let expected = "ActionParameterValue::String: test".to_string();
    assert_eq!(format!("{}", val_str), expected);

    let val_num = ActionParameterValue::Number(1.00);
    let expected = "ActionParameterValue::Number: 1.00".to_string();
    assert_eq!(format!("{}", val_num), expected);

    let val_int = ActionParameterValue::Integer(1);
    let expected = "ActionParameterValue::Integer: 1".to_string();
    assert_eq!(format!("{}", val_int), expected);

    let val_bool = ActionParameterValue::Boolean(true);
    let expected = "ActionParameterValue::Boolean: true".to_string();
    assert_eq!(format!("{}", val_bool), expected);

    let val_ctx = ActionParameterValue::ContextualLink(1);
    let expected = "ActionParameterValue::ContextualLink(1)".to_string();
    assert_eq!(format!("{}", val_ctx), expected);
}
