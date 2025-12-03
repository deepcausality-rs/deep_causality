/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{CausalityError, CausalityErrorEnum, EffectValue, PropagatingEffect};
use deep_causality_haft::LogSize;

#[test]
fn test_pure() {
    let effect = PropagatingEffect::pure(42);

    if let EffectValue::Value(v) = effect.value {
        assert_eq!(v, 42);
    } else {
        panic!("Expected Value(42)");
    }

    assert!(effect.error.is_none());
    assert!(effect.logs.is_empty());
}

#[test]
fn test_bind() {
    let initial = PropagatingEffect::pure(10);

    let next = PropagatingEffect::bind(initial, |val, _state, _ctx| {
        if let EffectValue::Value(v) = val {
            PropagatingEffect::pure(v + 1)
        } else {
            panic!("Expected Value");
        }
    });

    if let EffectValue::Value(v) = next.value {
        assert_eq!(v, 11);
    } else {
        panic!("Expected Value(11)");
    }
}

#[test]
fn test_bind_with_error() {
    let mut effect = PropagatingEffect::pure(10);
    effect.error = Some(CausalityError::new(CausalityErrorEnum::InternalLogicError));

    let next = PropagatingEffect::bind(effect, |val, _state, _ctx| {
        if let EffectValue::Value(v) = val {
            PropagatingEffect::pure(v + 1)
        } else {
            panic!("Expected Value");
        }
    });

    assert!(next.error.is_some());
}

#[test]
fn test_stateless_nature() {
    let effect = PropagatingEffect::pure(42);
    assert_eq!(effect.state, ());
    assert_eq!(effect.context, None);
}
