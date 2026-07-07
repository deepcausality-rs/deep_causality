/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{
    CausalEffect, CausalityError, CausalityErrorEnum, PropagatingEffect, PropagatingProcess,
};

#[test]
fn test_with_state() {
    let effect = PropagatingEffect::pure(42);
    let process = PropagatingProcess::with_state(effect, 100, None::<String>);

    if let Some(v) = process.value() {
        assert_eq!(*v, 42);
    } else {
        panic!("Expected Value(42)");
    }

    assert_eq!(*process.state(), 100);
    assert_eq!(*process.context(), None);
    assert!(process.error().is_none());
}

#[test]
fn test_with_state_and_context() {
    let effect = PropagatingEffect::pure(42);
    let process = PropagatingProcess::with_state(effect, 100, Some("test context".to_string()));

    assert_eq!(*process.state(), 100);
    assert_eq!(*process.context(), Some("test context".to_string()));
}

#[test]
fn test_bind_with_state() {
    let effect = PropagatingEffect::pure(10);
    let initial = PropagatingProcess::with_state(effect, 0, None::<String>);

    let next = initial.bind(|val, state, _ctx| {
        if let Some(v) = val.into_value() {
            let new_val = v + 1;
            let new_state = state + 1;
            PropagatingProcess::new(
                Ok(CausalEffect::value(new_val)),
                new_state,
                None,
                Default::default(),
            )
        } else {
            panic!("Expected Value");
        }
    });

    if let Some(v) = next.value() {
        assert_eq!(*v, 11);
    } else {
        panic!("Expected Value(11)");
    }

    assert_eq!(*next.state(), 1);
}

#[test]
fn test_error_propagation() {
    let process: PropagatingProcess<i32, i32, String> = PropagatingProcess::new(
        Err(CausalityError::new(CausalityErrorEnum::InternalLogicError)),
        0,
        None,
        Default::default(),
    );

    // The continuation is not invoked on an errored process (left zero).
    let next = process.bind(|val, state, _ctx| {
        let effect = PropagatingEffect::pure(if let Some(v) = val.into_value() {
            v + 1
        } else {
            0
        });
        PropagatingProcess::with_state(effect, state + 1, None)
    });

    assert!(next.is_err());
    assert_eq!(
        next.error(),
        Some(&CausalityError::new(CausalityErrorEnum::InternalLogicError))
    );
    // Error short-circuit preserves the state untouched.
    assert_eq!(*next.state(), 0);
}
