/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{CausalityError, EffectValue, NumericValue, PropagatingEffect};
use deep_causality_num::{Complex, Quaternion};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertainBool, MaybeUncertainF64, UncertainBool, UncertainF64,
};
use std::collections::HashMap;

#[test]
fn test_from_effect_value() {
    let effect = PropagatingEffect::from_effect_value(EffectValue::Boolean(true));
    assert!(matches!(effect.value, EffectValue::Boolean(true)));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_deterministic() {
    let effect = PropagatingEffect::from_boolean(true);
    assert!(matches!(effect.value, EffectValue::Boolean(true)));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_numerical() {
    let effect = PropagatingEffect::from_numerical(42.0);
    assert!(matches!(effect.value, EffectValue::Numerical(v) if v == 42.0));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_numeric() {
    let effect = PropagatingEffect::from_numeric(NumericValue::I64(42));
    assert!(matches!(
        effect.value,
        EffectValue::Number(NumericValue::I64(42))
    ));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_probabilistic() {
    let effect = PropagatingEffect::from_probabilistic(0.75);
    assert!(matches!(effect.value, EffectValue::Probabilistic(v) if v == 0.75));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_tensor() {
    let tensor = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let effect = PropagatingEffect::from_tensor(tensor.clone());
    assert!(matches!(effect.value, EffectValue::Tensor(ref t) if t == &tensor));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_complex() {
    let complex = Complex::new(1.0, 2.0);
    let effect = PropagatingEffect::from_complex(complex);
    assert!(matches!(effect.value, EffectValue::Complex(c) if c == complex));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_complex_tensor() {
    let complex_tensor = CausalTensor::new(vec![Complex::new(1.0, 2.0)], vec![1]).unwrap();
    let effect = PropagatingEffect::from_complex_tensor(complex_tensor.clone());
    assert!(matches!(effect.value, EffectValue::ComplexTensor(ref t) if t == &complex_tensor));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_quaternion() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let effect = PropagatingEffect::from_quaternion(q);
    assert!(matches!(effect.value, EffectValue::Quaternion(quat) if quat == q));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_quaternion_tensor() {
    let q_tensor = CausalTensor::new(vec![Quaternion::new(1.0, 2.0, 3.0, 4.0)], vec![1]).unwrap();
    let effect = PropagatingEffect::from_quaternion_tensor(q_tensor.clone());
    assert!(matches!(effect.value, EffectValue::QuaternionTensor(ref t) if t == &q_tensor));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_uncertain_bool() {
    let ub = UncertainBool::bernoulli(0.8);
    let effect = PropagatingEffect::from_uncertain_bool(ub);
    assert!(matches!(effect.value, EffectValue::UncertainBool(_)));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_uncertain_float() {
    let uf = UncertainF64::normal(0.5, 0.1);
    let effect = PropagatingEffect::from_uncertain_float(uf);
    assert!(matches!(effect.value, EffectValue::UncertainFloat(_)));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_maybe_uncertain_bool() {
    let mub = MaybeUncertainBool::from_value(true);
    let effect = PropagatingEffect::from_maybe_uncertain_bool(mub);
    assert!(matches!(effect.value, EffectValue::MaybeUncertainBool(_)));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_maybe_uncertain_float() {
    let muf = MaybeUncertainF64::from_value(0.5);
    let effect = PropagatingEffect::from_maybe_uncertain_float(muf);
    assert!(matches!(effect.value, EffectValue::MaybeUncertainFloat(_)));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_contextual_link() {
    let effect = PropagatingEffect::from_contextual_link(42, 42);
    assert!(matches!(effect.value, EffectValue::ContextualLink(42, 42)));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_map() {
    let map = HashMap::new();
    let effect = PropagatingEffect::from_map(map);
    assert!(matches!(effect.value, EffectValue::Map(_)));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_relay_to() {
    let inner_effect = Box::new(PropagatingEffect::from_boolean(true));
    let effect = PropagatingEffect::from_relay_to(1, inner_effect);
    assert!(matches!(effect.value, EffectValue::RelayTo(1, _)));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}

#[test]
fn test_from_error() {
    let err = CausalityError::new("test error".to_string());
    let effect = PropagatingEffect::from_error(err);
    assert!(matches!(effect.value, EffectValue::None));
    assert!(effect.is_err());
    assert_eq!(
        effect.error.as_ref().unwrap().to_string(),
        "CausalityError: test error"
    );
    assert!(!effect.has_log());
}

#[test]
fn test_none() {
    let effect = PropagatingEffect::none();
    assert!(matches!(effect.value, EffectValue::None));
    assert!(effect.is_ok());
    assert!(!effect.has_log());
}
