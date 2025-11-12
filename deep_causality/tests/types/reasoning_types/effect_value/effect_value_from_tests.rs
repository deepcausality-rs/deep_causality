/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{EffectValue, NumericValue, PropagatingValue};
use deep_causality_num::{Complex, Quaternion};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertainBool, MaybeUncertainF64, UncertainBool, UncertainF64,
};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
struct MyExternalType(i32);

impl Display for MyExternalType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyExternalType({})", self.0)
    }
}

#[test]
fn test_from_bool() {
    let val = true;
    let effect: EffectValue = val.into();
    assert_eq!(effect, EffectValue::Deterministic(true));
}

#[test]
fn test_from_numeric_value() {
    let val = NumericValue::I64(42);
    let effect: EffectValue = val.clone().into();
    assert_eq!(effect, EffectValue::Number(val));
}

#[test]
fn test_from_numerical_value() {
    let val = 42.0f64;
    let effect: EffectValue = val.into();
    assert_eq!(effect, EffectValue::Numerical(val));
}

#[test]
fn test_from_causal_tensor_f64() {
    let val = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let effect: EffectValue = val.clone().into();
    assert_eq!(effect, EffectValue::Tensor(val));
}

#[test]
fn test_from_complex_f64() {
    let val = Complex::new(1.0, 2.0);
    let effect: EffectValue = val.into();
    assert_eq!(effect, EffectValue::Complex(val));
}

#[test]
fn test_from_causal_tensor_complex_f64() {
    let val = CausalTensor::new(vec![Complex::new(1.0, 2.0)], vec![1]).unwrap();
    let effect: EffectValue = val.clone().into();
    assert_eq!(effect, EffectValue::ComplexTensor(val));
}

#[test]
fn test_from_quaternion_f64() {
    let val = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let effect: EffectValue = val.into();
    assert_eq!(effect, EffectValue::Quaternion(val));
}

#[test]
fn test_from_causal_tensor_quaternion_f64() {
    let val = CausalTensor::new(vec![Quaternion::new(1.0, 2.0, 3.0, 4.0)], vec![1]).unwrap();
    let effect: EffectValue = val.clone().into();
    assert_eq!(effect, EffectValue::QuaternionTensor(val));
}

#[test]
fn test_from_uncertain_bool() {
    let val = UncertainBool::bernoulli(0.8);
    let effect: EffectValue = val.clone().into();
    assert_eq!(effect, EffectValue::UncertainBool(val));
}

#[test]
fn test_from_uncertain_f64() {
    let val = UncertainF64::normal(0.5, 0.1);
    let effect: EffectValue = val.clone().into();
    assert_eq!(effect, EffectValue::UncertainFloat(val));
}

#[test]
fn test_from_maybe_uncertain_bool() {
    let val = MaybeUncertainBool::from_value(true);
    let effect: EffectValue = val.clone().into();
    assert_eq!(effect, EffectValue::MaybeUncertainBool(val));
}

#[test]
fn test_from_maybe_uncertain_f64() {
    let val = MaybeUncertainF64::from_value(0.5);
    let effect: EffectValue = val.clone().into();
    assert_eq!(effect, EffectValue::MaybeUncertainFloat(val));
}

#[test]
fn test_from_box_dyn_propagating_value() {
    let val: Box<dyn PropagatingValue> = Box::new(MyExternalType(42));
    let effect: EffectValue = val.into();
    assert!(effect.is_external());
    let extracted = effect.as_external().unwrap();
    assert_eq!(
        extracted.as_any().downcast_ref::<MyExternalType>().unwrap(),
        &MyExternalType(42)
    );
}
