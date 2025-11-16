/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{EffectValue, NumericValue, PropagatingEffect, PropagatingValue};
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
fn test_eq_none() {
    assert_eq!(EffectValue::None, EffectValue::None);
    assert_ne!(EffectValue::None, EffectValue::Deterministic(true));
}

#[test]
fn test_eq_deterministic() {
    assert_eq!(
        EffectValue::Deterministic(true),
        EffectValue::Deterministic(true)
    );
    assert_ne!(
        EffectValue::Deterministic(true),
        EffectValue::Deterministic(false)
    );
}

#[test]
fn test_eq_number() {
    assert_eq!(
        EffectValue::Number(NumericValue::I64(42)),
        EffectValue::Number(NumericValue::I64(42))
    );
    assert_ne!(
        EffectValue::Number(NumericValue::I64(42)),
        EffectValue::Number(NumericValue::I64(43))
    );
}

#[test]
fn test_eq_numerical() {
    assert_eq!(EffectValue::Numerical(42.0), EffectValue::Numerical(42.0));
    assert_ne!(EffectValue::Numerical(42.0), EffectValue::Numerical(43.0));
}

#[test]
fn test_eq_probabilistic() {
    assert_eq!(
        EffectValue::Probabilistic(0.5),
        EffectValue::Probabilistic(0.5)
    );
    assert_ne!(
        EffectValue::Probabilistic(0.5),
        EffectValue::Probabilistic(0.6)
    );
}

#[test]
fn test_eq_tensor() {
    let t1 = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let t2 = CausalTensor::new(vec![2.0], vec![1]).unwrap();
    assert_eq!(
        EffectValue::Tensor(t1.clone()),
        EffectValue::Tensor(t1.clone())
    );
    assert_ne!(EffectValue::Tensor(t1), EffectValue::Tensor(t2));
}

#[test]
fn test_eq_complex() {
    let c1 = Complex::new(1.0, 2.0);
    let c2 = Complex::new(3.0, 4.0);
    assert_eq!(EffectValue::Complex(c1), EffectValue::Complex(c1));
    assert_ne!(EffectValue::Complex(c1), EffectValue::Complex(c2));
}

#[test]
fn test_eq_complex_tensor() {
    let t1 = CausalTensor::new(vec![Complex::new(1.0, 2.0)], vec![1]).unwrap();
    let t2 = CausalTensor::new(vec![Complex::new(3.0, 4.0)], vec![1]).unwrap();
    assert_eq!(
        EffectValue::ComplexTensor(t1.clone()),
        EffectValue::ComplexTensor(t1.clone())
    );
    assert_ne!(
        EffectValue::ComplexTensor(t1),
        EffectValue::ComplexTensor(t2)
    );
}

#[test]
fn test_eq_quaternion() {
    let q1 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let q2 = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    assert_eq!(EffectValue::Quaternion(q1), EffectValue::Quaternion(q1));
    assert_ne!(EffectValue::Quaternion(q1), EffectValue::Quaternion(q2));
}

#[test]
fn test_eq_quaternion_tensor() {
    let t1 = CausalTensor::new(vec![Quaternion::new(1.0, 2.0, 3.0, 4.0)], vec![1]).unwrap();
    let t2 = CausalTensor::new(vec![Quaternion::new(5.0, 6.0, 7.0, 8.0)], vec![1]).unwrap();
    assert_eq!(
        EffectValue::QuaternionTensor(t1.clone()),
        EffectValue::QuaternionTensor(t1.clone())
    );
    assert_ne!(
        EffectValue::QuaternionTensor(t1),
        EffectValue::QuaternionTensor(t2)
    );
}

#[test]
fn test_eq_uncertain_bool() {
    let ub1 = UncertainBool::bernoulli(0.8);
    let ub2 = UncertainBool::bernoulli(0.9);
    assert_eq!(
        EffectValue::UncertainBool(ub1.clone()),
        EffectValue::UncertainBool(ub1.clone())
    );
    assert_ne!(
        EffectValue::UncertainBool(ub1),
        EffectValue::UncertainBool(ub2)
    );
}

#[test]
fn test_eq_uncertain_float() {
    let uf1 = UncertainF64::normal(0.5, 0.1);
    let uf2 = UncertainF64::normal(0.6, 0.2);
    assert_eq!(
        EffectValue::UncertainFloat(uf1.clone()),
        EffectValue::UncertainFloat(uf1.clone())
    );
    assert_ne!(
        EffectValue::UncertainFloat(uf1),
        EffectValue::UncertainFloat(uf2)
    );
}

#[test]
fn test_eq_maybe_uncertain_bool() {
    let mub1 = MaybeUncertainBool::from_value(true);
    let mub2 = MaybeUncertainBool::from_value(false);
    assert_eq!(
        EffectValue::MaybeUncertainBool(mub1.clone()),
        EffectValue::MaybeUncertainBool(mub1.clone())
    );
    assert_ne!(
        EffectValue::MaybeUncertainBool(mub1),
        EffectValue::MaybeUncertainBool(mub2)
    );
}

#[test]
fn test_eq_maybe_uncertain_float() {
    let muf1 = MaybeUncertainF64::from_value(0.5);
    let muf2 = MaybeUncertainF64::from_value(0.6);
    assert_eq!(
        EffectValue::MaybeUncertainFloat(muf1.clone()),
        EffectValue::MaybeUncertainFloat(muf1.clone())
    );
    assert_ne!(
        EffectValue::MaybeUncertainFloat(muf1),
        EffectValue::MaybeUncertainFloat(muf2)
    );
}

#[test]
fn test_eq_contextual_link() {
    assert_eq!(
        EffectValue::ContextualLink(42, 42),
        EffectValue::ContextualLink(42, 42)
    );
    assert_ne!(
        EffectValue::ContextualLink(42, 42),
        EffectValue::ContextualLink(43, 43)
    );
}

#[test]
fn test_eq_map() {
    let mut map1 = HashMap::new();
    map1.insert(1, Box::new(PropagatingEffect::from_deterministic(true)));
    let mut map2 = HashMap::new();
    map2.insert(2, Box::new(PropagatingEffect::from_deterministic(false)));

    assert_eq!(
        EffectValue::Map(map1.clone()),
        EffectValue::Map(map1.clone())
    );
    assert_ne!(EffectValue::Map(map1), EffectValue::Map(map2));
}

#[test]
fn test_eq_relay_to() {
    let effect1 = Box::new(PropagatingEffect::from_deterministic(true));
    let effect2 = Box::new(PropagatingEffect::from_deterministic(false));

    assert_eq!(
        EffectValue::RelayTo(1, effect1.clone()),
        EffectValue::RelayTo(1, effect1.clone())
    );
    assert_ne!(
        EffectValue::RelayTo(1, effect1.clone()),
        EffectValue::RelayTo(2, effect1.clone())
    );
    assert_ne!(
        EffectValue::RelayTo(1, effect1),
        EffectValue::RelayTo(1, effect2)
    );
}

#[test]
fn test_eq_external() {
    let val1: Box<dyn PropagatingValue> = Box::new(MyExternalType(42));
    let val2: Box<dyn PropagatingValue> = Box::new(MyExternalType(43));

    assert_eq!(
        EffectValue::External(val1.clone()),
        EffectValue::External(val1.clone())
    );
    assert_ne!(EffectValue::External(val1), EffectValue::External(val2));
}
