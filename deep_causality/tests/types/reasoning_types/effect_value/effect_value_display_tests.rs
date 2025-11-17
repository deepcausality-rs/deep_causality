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
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
struct MyExternalType(i32);

impl Display for MyExternalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyExternalType({})", self.0)
    }
}

// PropagatingValue is automatically implemented for types that are
// Debug + Display + Clone + PartialEq + Send + Sync + 'static.

#[test]
fn test_display_none() {
    let value = EffectValue::None;
    assert_eq!(value.to_string(), "None");
}

#[test]
fn test_display_deterministic() {
    let value = EffectValue::Boolean(true);
    assert_eq!(value.to_string(), "Deterministic(true)");
}

#[test]
fn test_display_number() {
    let value = EffectValue::Number(NumericValue::I64(42));
    assert_eq!(value.to_string(), "Number(I64(42))");
}

#[test]
fn test_display_numerical() {
    let value = EffectValue::Numerical(42.0);
    assert_eq!(value.to_string(), "Numerical(42)");
}

#[test]
fn test_display_probabilistic() {
    let value = EffectValue::Probabilistic(0.75);
    assert_eq!(value.to_string(), "Probabilistic(0.75)");
}

#[test]
fn test_display_tensor() {
    let tensor = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let value = EffectValue::Tensor(tensor.clone());
    assert_eq!(value.to_string(), format!("Tensor({:?})", tensor));
}

#[test]
fn test_display_complex() {
    let complex = Complex::new(1.0, 2.0);
    let value = EffectValue::Complex(complex);
    assert_eq!(value.to_string(), format!("Complex({:?})", complex));
}

#[test]
fn test_display_complex_tensor() {
    let complex_tensor = CausalTensor::new(vec![Complex::new(1.0, 2.0)], vec![1]).unwrap();
    let value = EffectValue::ComplexTensor(complex_tensor.clone());
    assert_eq!(
        value.to_string(),
        format!("ComplexTensor({:?})", complex_tensor)
    );
}

#[test]
fn test_display_quaternion() {
    let quaternion = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let value = EffectValue::Quaternion(quaternion);
    assert_eq!(value.to_string(), format!("Quaternion({:?})", quaternion));
}

#[test]
fn test_display_quaternion_tensor() {
    let quaternion_tensor =
        CausalTensor::new(vec![Quaternion::new(1.0, 2.0, 3.0, 4.0)], vec![1]).unwrap();
    let value = EffectValue::QuaternionTensor(quaternion_tensor.clone());
    assert_eq!(
        value.to_string(),
        format!("QuaternionTensor({:?})", quaternion_tensor)
    );
}

#[test]
fn test_display_uncertain_bool() {
    let ub = UncertainBool::bernoulli(0.8);
    let value = EffectValue::UncertainBool(ub.clone());
    assert_eq!(value.to_string(), format!("UncertainBool({:?})", ub));
}

#[test]
fn test_display_uncertain_float() {
    let uf = UncertainF64::normal(0.5, 0.1);
    let value = EffectValue::UncertainFloat(uf.clone());
    assert_eq!(value.to_string(), format!("UncertainFloat({:?})", uf));
}

#[test]
fn test_display_maybe_uncertain_bool() {
    let mub = MaybeUncertainBool::from_bernoulli_and_uncertain(0.9, UncertainBool::bernoulli(0.7));
    let value = EffectValue::MaybeUncertainBool(mub.clone());
    assert_eq!(value.to_string(), format!("MaybeUncertainBool({:?})", mub));
}

#[test]
fn test_display_maybe_uncertain_float() {
    let muf = MaybeUncertainF64::from_bernoulli_and_uncertain(0.9, UncertainF64::normal(0.5, 0.1));
    let value = EffectValue::MaybeUncertainFloat(muf.clone());
    assert_eq!(value.to_string(), format!("MaybeUncertainFloat({:?})", muf));
}

#[test]
fn test_display_contextual_link() {
    let value = EffectValue::ContextualLink(42, 42);
    assert_eq!(
        value.to_string(),
        "ContextualLink(ContextoidId: 42, ContextoidId: 42)"
    );
}

#[test]
fn test_display_map() {
    let mut map = HashMap::new();
    map.insert(1, Box::new(PropagatingEffect::from_boolean(true)));
    let value = EffectValue::Map(map.clone());
    assert_eq!(value.to_string(), format!("Map({:?})", map));
}

#[test]
fn test_display_relay_to() {
    let effect = Box::new(PropagatingEffect::from_boolean(true));
    let value = EffectValue::RelayTo(12, effect.clone());
    assert_eq!(
        value.to_string(),
        format!("RelayTo(target: 12, value: {})", effect)
    );
}

#[test]
fn test_display_external() {
    let external = MyExternalType(123);
    let value = EffectValue::External(Box::new(external.clone()));
    assert_eq!(value.to_string(), format!("External({:?})", external));
}
