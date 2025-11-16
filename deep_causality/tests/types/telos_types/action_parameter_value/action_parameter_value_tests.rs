/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{ActionParameterValue, EffectValue, PropagatingEffect};
use deep_causality_num::Complex;
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{UncertainBool, UncertainF64};

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
fn test_action_parameter_value_contextual_link() {
    let val = ActionParameterValue::ContextualLink(1, 1);
    assert_eq!(val, ActionParameterValue::ContextualLink(1, 1));
    assert_ne!(val, ActionParameterValue::ContextualLink(2, 2));
    assert_eq!(
        format!("{}", val),
        "ActionParameterValue::ContextualLink(1, 1)"
    );
    assert_eq!(format!("{:?}", val), "ContextualLink(1, 1)");
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

    let val_ctx = ActionParameterValue::ContextualLink(1, 1);
    let cloned_ctx = val_ctx.clone();
    assert_eq!(val_ctx, cloned_ctx);
}

#[test]
fn test_action_parameter_value_inequality_across_variants() {
    let val_str = ActionParameterValue::String("test".to_string());
    let val_num = ActionParameterValue::Number(1.0);
    let val_int = ActionParameterValue::Integer(1);
    let val_bool = ActionParameterValue::Boolean(true);
    let val_ctx = ActionParameterValue::ContextualLink(1, 1);

    assert_ne!(val_str, val_num);
    assert_ne!(val_str, val_int);
    assert_ne!(val_str, val_bool);
    assert_ne!(val_str, val_ctx);

    assert_ne!(val_num, val_int);
    assert_ne!(val_num, val_bool);
    assert_ne!(val_num, val_ctx);

    assert_ne!(val_int, val_bool);
    assert_ne!(val_int, val_ctx);

    assert_ne!(val_bool, val_ctx);
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

    let val_ctx = ActionParameterValue::ContextualLink(1, 1);
    let expected = "ActionParameterValue::ContextualLink(1, 1)".to_string();
    assert_eq!(format!("{}", val_ctx), expected);
}

#[test]
fn test_from_effect_value() {
    // Deterministic
    let effect_val = EffectValue::Deterministic(true);
    let action_param_val: ActionParameterValue = effect_val.into();
    assert_eq!(action_param_val, ActionParameterValue::Boolean(true));

    // Numerical
    let effect_val = EffectValue::Numerical(123.45);
    let action_param_val: ActionParameterValue = effect_val.into();
    assert_eq!(action_param_val, ActionParameterValue::Number(123.45));

    // Probabilistic
    let effect_val = EffectValue::Probabilistic(0.75);
    let action_param_val: ActionParameterValue = effect_val.into();
    assert_eq!(action_param_val, ActionParameterValue::Number(0.75));

    // UncertainBool
    let effect_val = EffectValue::UncertainBool(UncertainBool::point(true));
    let action_param_val: ActionParameterValue = effect_val.into();
    assert_eq!(action_param_val, ActionParameterValue::Boolean(true));

    // UncertainFloat
    let effect_val = EffectValue::UncertainFloat(UncertainF64::point(10.5));
    let action_param_val: ActionParameterValue = effect_val.into();
    assert_eq!(action_param_val, ActionParameterValue::Number(10.5));

    // CausalTensor
    let effect_val = EffectValue::Tensor(CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap());
    let action_param_val: ActionParameterValue = effect_val.into();
    assert!(matches!(action_param_val, ActionParameterValue::String(_)));
    if let ActionParameterValue::String(s) = action_param_val {
        assert!(s.contains("Tensor"));
    }

    // Complex
    let effect_val = EffectValue::Complex(Complex::new(1.0, 2.0));
    let action_param_val: ActionParameterValue = effect_val.into();
    assert!(matches!(action_param_val, ActionParameterValue::String(_)));
    if let ActionParameterValue::String(s) = action_param_val {
        assert!(s.contains("Complex"));
    }

    // ContextualLink
    let effect_val = EffectValue::ContextualLink(42, 42);
    let action_param_val: ActionParameterValue = effect_val.into();
    assert_eq!(
        action_param_val,
        ActionParameterValue::ContextualLink(42, 42)
    );

    // None
    let effect_val = EffectValue::None;
    let action_param_val: ActionParameterValue = effect_val.into();
    assert_eq!(
        action_param_val,
        ActionParameterValue::String("None".to_string())
    );

    // Unsupported
    let propagating_effect_val = PropagatingEffect::from_deterministic(true);
    let effect_val = EffectValue::RelayTo(32, Box::new(propagating_effect_val));
    let action_param_val: ActionParameterValue = effect_val.into();
    assert_eq!(
        action_param_val,
        ActionParameterValue::String("Unsupported EffectValue".to_string())
    );
}
