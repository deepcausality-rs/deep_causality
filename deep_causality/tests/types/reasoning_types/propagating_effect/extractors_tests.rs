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
fn test_as_bool() {
    let effect1 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect1.as_bool(), Some(true));

    let effect2 = PropagatingEffect::Deterministic(false);
    assert_eq!(effect2.as_bool(), Some(false));

    let effect3 = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(effect3.as_bool(), None);

    let point = Uncertain::<bool>::point(true);
    let effect4 = UncertainBool(point);
    assert_eq!(effect4.as_bool(), None);

    let point = Uncertain::<f64>::point(4.0f64);
    let effect5 = UncertainFloat(point);
    assert_eq!(effect5.as_bool(), None);

    let effect6 = PropagatingEffect::ContextualLink(1, 1);
    assert_eq!(effect6.as_bool(), None);
}

#[test]
fn test_as_numerical() {
    let effect1 = PropagatingEffect::from_numerical(0.5);
    assert_eq!(effect1.as_numerical(), Some(0.5));

    let effect2 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect2.as_numerical(), None);
    
    let effect3 = PropagatingEffect::ContextualLink(1, 1);
    assert_eq!(effect3.as_numerical(), None);
}

#[test]
fn test_as_probability() {
    let effect1 = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(effect1.as_probability(), Some(0.5));

    let effect2 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect2.as_probability(), None);

    let effect3 = PropagatingEffect::ContextualLink(1, 1);
    assert_eq!(effect3.as_probability(), None);

    let point = Uncertain::<bool>::point(true);
    let effect4 = UncertainBool(point);
    assert_eq!(effect4.as_probability(), None);

    let point = Uncertain::<f64>::point(4.0f64);
    let effect5 = UncertainFloat(point);
    assert_eq!(effect5.as_probability(), None);
}

#[test]
fn test_as_tensor() {
    let res = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]);
    assert!(res.is_ok());
    let tensor = res.unwrap();

    let effect1 = PropagatingEffect::Tensor(tensor.clone());
    assert!(effect1.is_tensor());
    assert_eq!(effect1.as_tensor(), Some(tensor));

    let effect2 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect2.as_tensor(), None);
}

#[test]
fn test_as_uncertain_bool() {
    let point = Uncertain::<bool>::point(true);
    let effect1 = UncertainBool(point.clone());
    assert_eq!(effect1.as_uncertain_bool(), Some(point));

    let effect2 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect2.as_uncertain_bool(), None);

    let effect3 = PropagatingEffect::ContextualLink(1, 1);
    assert_eq!(effect3.as_uncertain_bool(), None);
}

#[test]
fn test_as_uncertain_float() {
    let point = Uncertain::<f64>::point(1.0f64);
    let effect1 = UncertainFloat(point.clone());
    assert_eq!(effect1.as_uncertain_float(), Some(point));

    let effect2 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect2.as_uncertain_float(), None);
}

#[test]
fn test_as_maybe_uncertain_bool() {
    let point = MaybeUncertain::<bool>::from_value(true);
    let effect1 = MaybeUncertainBool(point.clone());
    assert!(effect1.is_maybe_uncertain_bool());
    assert_eq!(effect1.as_maybe_uncertain_bool(), Some(point));

    let effect2 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect2.as_maybe_uncertain_bool(), None);
}

#[test]
fn test_as_maybe_uncertain_float() {
    let point = MaybeUncertain::<f64>::from_value(4.0f64);
    let effect1 = MaybeUncertainFloat(point.clone());
    assert!(effect1.is_maybe_uncertain_float());
    assert_eq!(effect1.as_maybe_uncertain_float(), Some(point));

    let effect2 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect2.as_maybe_uncertain_float(), None);
}

#[test]
fn test_as_contextual_link() {
    let effect1 = PropagatingEffect::ContextualLink(1, 2);
    assert_eq!(effect1.as_contextual_link(), Some((1, 2)));

    let effect2 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect2.as_contextual_link(), None);

    let effect3 = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(effect3.as_contextual_link(), None);
}
