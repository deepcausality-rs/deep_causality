/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::PropagatingEffect;
use deep_causality::PropagatingEffect::{
    MaybeUncertainBool, MaybeUncertainFloat, UncertainBool, UncertainFloat,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{MaybeUncertain, Uncertain};

#[test]
fn test_is_deterministic() {
    let effect1 = PropagatingEffect::Deterministic(true);
    assert!(effect1.is_deterministic());

    let effect2 = PropagatingEffect::Probabilistic(0.5);
    assert!(!effect2.is_deterministic());

    let effect3 = PropagatingEffect::ContextualLink(1, 1);
    assert!(!effect3.is_deterministic());
}

#[test]
fn test_is_numerical() {
    let effect1 = PropagatingEffect::from_numerical(0.5);
    assert!(effect1.is_numerical());
}

#[test]
fn test_is_probabilistic() {
    let effect2 = PropagatingEffect::Probabilistic(0.5);
    assert!(effect2.is_probabilistic());

    let effect3 = PropagatingEffect::ContextualLink(1, 1);
    assert!(!effect3.is_probabilistic());
}

#[test]
fn test_is_tensor() {
    let res = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]);
    assert!(res.is_ok());
    let tensor = res.unwrap();

    let effect1 = PropagatingEffect::Tensor(tensor);
    assert!(effect1.is_tensor());

    // Ensure its not float
    assert!(!effect1.is_uncertain_float());
}

#[test]
fn test_is_complex_tensor() {
    use deep_causality_num::Complex;
    let res = CausalTensor::new(vec![Complex::new(1.0, 2.0)], vec![1]);
    assert!(res.is_ok());
    let complex_tensor = res.unwrap();

    let effect1 = PropagatingEffect::ComplexTensor(complex_tensor);
    assert!(effect1.is_complex_tensor());

    assert!(!effect1.is_tensor());
}

#[test]
fn test_is_uncertain_bool() {
    let point = Uncertain::<bool>::point(true);
    let effect1 = UncertainBool(point);
    assert!(effect1.is_uncertain_bool());

    // Ensure its not float
    assert!(!effect1.is_uncertain_float());
}

#[test]
fn test_is_uncertain_float() {
    let point = Uncertain::<f64>::point(4.0f64);
    let effect1 = UncertainFloat(point);
    assert!(effect1.is_uncertain_float());

    // Ensure its not bool
    assert!(!effect1.is_uncertain_bool());
}

#[test]
fn test_is_maybe_uncertain_bool() {
    let point = MaybeUncertain::<bool>::from_value(true);
    let effect1 = MaybeUncertainBool(point);
    assert!(effect1.is_maybe_uncertain_bool());

    let point = MaybeUncertain::<bool>::always_none();
    let effect1 = MaybeUncertainBool(point);
    assert!(effect1.is_maybe_uncertain_bool());

    assert!(!effect1.is_maybe_uncertain_float());
}

#[test]
fn test_is_maybe_uncertain_float() {
    let point = MaybeUncertain::<f64>::from_value(4.0f64);
    let effect1 = MaybeUncertainFloat(point);
    assert!(effect1.is_maybe_uncertain_float());

    let point = MaybeUncertain::<f64>::always_none();
    let effect1 = MaybeUncertainFloat(point);
    assert!(effect1.is_maybe_uncertain_float());

    assert!(!effect1.is_uncertain_bool());
}

#[test]
fn test_is_contextual_link() {
    let effect1 = PropagatingEffect::Deterministic(true);
    assert!(!effect1.is_contextual_link());

    let effect2 = PropagatingEffect::Probabilistic(0.5);
    assert!(!effect2.is_contextual_link());

    let effect3 = PropagatingEffect::ContextualLink(1, 1);
    assert!(effect3.is_contextual_link());
}
