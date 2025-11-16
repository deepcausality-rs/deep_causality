/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{EffectValue, NumericValue, PropagatingEffect};
use deep_causality_num::{Complex, Quaternion};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertainBool, MaybeUncertainF64, UncertainBool, UncertainF64,
};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
struct MyExternalType(i32);

impl Display for MyExternalType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyExternalType({})", self.0)
    }
}

#[test]
fn test_is_none() {
    let effect = EffectValue::None;
    assert!(effect.is_none());
    assert!(!effect.is_deterministic()); // Check against another variant
}

#[test]
fn test_is_deterministic() {
    let effect = EffectValue::Deterministic(true);
    assert!(effect.is_deterministic());
    assert!(!effect.is_none());
}

#[test]
fn test_is_number() {
    let effect = EffectValue::Number(NumericValue::I64(42));
    assert!(effect.is_number());
    assert!(!effect.is_none());
}

#[test]
fn test_is_numerical() {
    let effect = EffectValue::Numerical(42.0);
    assert!(effect.is_numeric());
    assert!(!effect.is_none());
}

#[test]
fn test_is_probabilistic() {
    let effect = EffectValue::Probabilistic(0.5);
    assert!(effect.is_probabilistic());
    assert!(!effect.is_none());
}

#[test]
fn test_is_tensor() {
    let tensor = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let effect = EffectValue::Tensor(tensor);
    assert!(effect.is_tensor());
    assert!(!effect.is_none());
}

#[test]
fn test_is_complex() {
    let complex = Complex::new(1.0, 2.0);
    let effect = EffectValue::Complex(complex);
    assert!(effect.is_complex());
    assert!(!effect.is_none());
}

#[test]
fn test_is_complex_tensor() {
    let tensor = CausalTensor::new(vec![Complex::new(1.0, 2.0)], vec![1]).unwrap();
    let effect = EffectValue::ComplexTensor(tensor);
    assert!(effect.is_complex_tensor());
    assert!(!effect.is_none());
}

#[test]
fn test_is_quaternion() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let effect = EffectValue::Quaternion(q);
    assert!(effect.is_quaternion());
    assert!(!effect.is_none());
}

#[test]
fn test_is_quaternion_tensor() {
    let tensor = CausalTensor::new(vec![Quaternion::new(1.0, 2.0, 3.0, 4.0)], vec![1]).unwrap();
    let effect = EffectValue::QuaternionTensor(tensor);
    assert!(effect.is_quaternion_tensor());
    assert!(!effect.is_none());
}

#[test]
fn test_is_uncertain_bool() {
    let ub = UncertainBool::bernoulli(0.8);
    let effect = EffectValue::UncertainBool(ub);
    assert!(effect.is_uncertain_bool());
    assert!(!effect.is_none());
}

#[test]
fn test_is_uncertain_float() {
    let uf = UncertainF64::normal(0.5, 0.1);
    let effect = EffectValue::UncertainFloat(uf);
    assert!(effect.is_uncertain_float());
    assert!(!effect.is_none());
}

#[test]
fn test_is_maybe_uncertain_bool() {
    let mub = MaybeUncertainBool::from_value(true);
    let effect = EffectValue::MaybeUncertainBool(mub);
    assert!(effect.is_maybe_uncertain_bool());
    assert!(!effect.is_none());
}

#[test]
fn test_is_maybe_uncertain_float() {
    let muf = MaybeUncertainF64::from_value(0.5);
    let effect = EffectValue::MaybeUncertainFloat(muf);
    assert!(effect.is_maybe_uncertain_float());
    assert!(!effect.is_none());
}

#[test]
fn test_is_contextual_link() {
    let effect = EffectValue::ContextualLink(42, 42);
    assert!(effect.is_contextual_link());
    assert!(!effect.is_none());
}

#[test]
fn test_is_map() {
    let mut map = HashMap::new();
    map.insert(1, Box::new(PropagatingEffect::from_deterministic(true)));
    let effect = EffectValue::Map(map);
    assert!(effect.is_map());
    assert!(!effect.is_none());
}

#[test]
fn test_is_relay_to() {
    let effect = EffectValue::RelayTo(12, Box::new(PropagatingEffect::from_deterministic(true)));
    assert!(effect.is_relay_to());
    assert!(!effect.is_none());
}

#[test]
fn test_is_external() {
    let external_val = MyExternalType(123);
    let effect = EffectValue::External(Box::new(external_val));
    assert!(effect.is_external());
    assert!(!effect.is_none());
}
