/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{
    CausalityError, CausalityErrorEnum, EffectValue, PropagatingEffect, PropagatingProcess,
};

#[test]
fn test_with_state() {
    let effect = PropagatingEffect::pure(42);
    let process = PropagatingProcess::with_state(effect, 100, None::<String>);

    if let EffectValue::Value(v) = process.value {
        assert_eq!(v, 42);
    } else {
        panic!("Expected Value(42)");
    }

    assert_eq!(process.state, 100);
    assert_eq!(process.context, None);
    assert!(process.error.is_none());
}

#[test]
fn test_with_state_and_context() {
    let effect = PropagatingEffect::pure(42);
    let process = PropagatingProcess::with_state(effect, 100, Some("test context".to_string()));

    assert_eq!(process.state, 100);
    assert_eq!(process.context, Some("test context".to_string()));
}

#[test]
fn test_bind_with_state() {
    let effect = PropagatingEffect::pure(10);
    let initial = PropagatingProcess::with_state(effect, 0, None::<String>);

    let next = initial.bind(|val, state, _ctx| {
        if let EffectValue::Value(v) = val {
            let new_val = v + 1;
            let new_state = state + 1;
            PropagatingProcess {
                value: EffectValue::Value(new_val),
                state: new_state,
                context: None,
                error: None,
                logs: Default::default(),
            }
        } else {
            panic!("Expected Value");
        }
    });

    if let EffectValue::Value(v) = next.value {
        assert_eq!(v, 11);
    } else {
        panic!("Expected Value(11)");
    }

    assert_eq!(next.state, 1);
}

#[test]
fn test_error_propagation() {
    let effect = PropagatingEffect::pure(10);
    let mut process = PropagatingProcess::with_state(effect, 0, None::<String>);
    process.error = Some(CausalityError::new(CausalityErrorEnum::InternalLogicError));

    let next = process.bind(|val, state, _ctx| {
        let effect = PropagatingEffect::pure(if let EffectValue::Value(v) = val {
            v + 1
        } else {
            0
        });
        PropagatingProcess::with_state(effect, state + 1, None)
    });

    assert!(next.error.is_some());
}
