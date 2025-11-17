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

macro_rules! test_extractor {
    ($test_name:ident, $variant:ident, $value:expr, $extractor:ident, $expected:expr) => {
        #[test]
        fn $test_name() {
            let effect = EffectValue::$variant($value);
            assert_eq!(effect.$extractor(), Some($expected));

            let non_effect = EffectValue::None;
            assert_eq!(non_effect.$extractor(), None);
        }
    };
}

test_extractor!(test_as_bool, Boolean, true, as_bool, true);
test_extractor!(
    test_as_probabilistic,
    Probabilistic,
    0.5,
    as_probabilistic,
    0.5
);

#[test]
fn test_as_number() {
    let num = NumericValue::I64(42);
    let effect = EffectValue::Number(num.clone());
    assert_eq!(effect.as_number(), Some(&num));
    let non_effect = EffectValue::None;
    assert_eq!(non_effect.as_number(), None);
}

#[test]
fn test_as_numerical() {
    let num = 42.0;
    let effect = EffectValue::Numerical(num);
    assert_eq!(effect.as_numerical(), Some(&num));
    let non_effect = EffectValue::None;
    assert_eq!(non_effect.as_numerical(), None);
}

#[test]
fn test_as_tensor() {
    let tensor = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let effect = EffectValue::Tensor(tensor.clone());
    assert_eq!(effect.as_tensor(), Some(&tensor));
    let non_effect = EffectValue::None;
    assert_eq!(non_effect.as_tensor(), None);
}

#[test]
fn test_as_complex() {
    let complex = Complex::new(1.0, 2.0);
    let effect = EffectValue::Complex(complex);
    assert_eq!(effect.as_complex(), Some(&complex));
    let non_effect = EffectValue::None;
    assert_eq!(non_effect.as_complex(), None);
}

#[test]
fn test_as_complex_tensor() {
    let tensor = CausalTensor::new(vec![Complex::new(1.0, 2.0)], vec![1]).unwrap();
    let effect = EffectValue::ComplexTensor(tensor.clone());
    assert_eq!(effect.as_complex_tensor(), Some(&tensor));
    let non_effect = EffectValue::None;
    assert_eq!(non_effect.as_complex_tensor(), None);
}

#[test]
fn test_as_quaternion() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let effect = EffectValue::Quaternion(q);
    assert_eq!(effect.as_quaternion(), Some(&q));
    let non_effect = EffectValue::None;
    assert_eq!(non_effect.as_quaternion(), None);
}

#[test]
fn test_as_quaternion_tensor() {
    let tensor = CausalTensor::new(vec![Quaternion::new(1.0, 2.0, 3.0, 4.0)], vec![1]).unwrap();
    let effect = EffectValue::QuaternionTensor(tensor.clone());
    assert_eq!(effect.as_quaternion_tensor(), Some(&tensor));
    let non_effect = EffectValue::None;
    assert_eq!(non_effect.as_quaternion_tensor(), None);
}

#[test]
fn test_as_uncertain_bool() {
    let ub = UncertainBool::bernoulli(0.8);
    let effect = EffectValue::UncertainBool(ub.clone());
    assert_eq!(effect.as_uncertain_bool(), Some(&ub));
    let non_effect = EffectValue::None;
    assert_eq!(non_effect.as_uncertain_bool(), None);
}

#[test]
fn test_as_uncertain_float() {
    let uf = UncertainF64::normal(0.5, 0.1);
    let effect = EffectValue::UncertainFloat(uf.clone());
    assert_eq!(effect.as_uncertain_float(), Some(&uf));
    let non_effect = EffectValue::None;
    assert_eq!(non_effect.as_uncertain_float(), None);
}

#[test]
fn test_as_maybe_uncertain_bool() {
    let mub = MaybeUncertainBool::from_value(true);
    let effect = EffectValue::MaybeUncertainBool(mub.clone());
    assert_eq!(effect.as_maybe_uncertain_bool(), Some(&mub));
    let non_effect = EffectValue::None;
    assert_eq!(non_effect.as_maybe_uncertain_bool(), None);
}

#[test]
fn test_as_maybe_uncertain_float() {
    let muf = MaybeUncertainF64::from_value(0.5);
    let effect = EffectValue::MaybeUncertainFloat(muf.clone());
    assert_eq!(effect.as_maybe_uncertain_float(), Some(&muf));
    let non_effect = EffectValue::None;
    assert_eq!(non_effect.as_maybe_uncertain_float(), None);
}

#[test]
fn test_as_contextual_link() {
    let id = 42;
    let effect = EffectValue::ContextualLink(id, id);
    assert_eq!(effect.as_contextual_link(), Some((&id, &id)));
    let non_effect = EffectValue::None;
    assert_eq!(non_effect.as_contextual_link(), None);
}

#[test]
fn test_as_map() {
    let mut map = HashMap::new();
    map.insert(1, Box::new(PropagatingEffect::from_boolean(true)));
    let effect = EffectValue::Map(map.clone());
    assert_eq!(effect.as_map(), Some(&map));
    let non_effect = EffectValue::None;
    assert_eq!(non_effect.as_map(), None);
}

#[test]
fn test_as_relay_to() {
    let target = 42;
    let inner_effect = PropagatingEffect::from_boolean(true);
    let effect = EffectValue::RelayTo(target, Box::new(inner_effect.clone()));
    assert_eq!(effect.as_relay_to(), Some((&target, &inner_effect)));
    let non_effect = EffectValue::None;
    assert_eq!(non_effect.as_relay_to(), None);
}

#[test]
fn test_as_external_and_try_from() {
    let external_val = MyExternalType(123);
    let effect = EffectValue::External(Box::new(external_val.clone()));

    // Test as_external
    let external_trait_obj = effect.as_external().unwrap();
    assert!(external_trait_obj.as_any().is::<MyExternalType>());

    // Test try_from_effect_value (happy path)
    let extracted: Option<&MyExternalType> = EffectValue::try_from_effect_value(&effect);
    assert_eq!(extracted, Some(&external_val));

    // Test try_from_effect_value (sad path)
    let non_effect = EffectValue::None;
    let extracted_none: Option<&MyExternalType> = EffectValue::try_from_effect_value(&non_effect);
    assert_eq!(extracted_none, None);
}
