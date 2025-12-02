/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{CausalMonad, EffectValue, Intervenable};
use deep_causality_haft::{LogAddEntry, LogSize, MonadEffect5};

#[test]
fn test_pure() {
    let process = CausalMonad::<i32, String>::pure(42);

    if let EffectValue::Value(v) = process.value {
        assert_eq!(v, 42);
    } else {
        panic!("Expected Value(42)");
    }

    assert_eq!(process.state, 0); // Default for i32
    assert!(process.context.is_none());
    assert!(process.error.is_none());
    assert!(process.logs.is_empty());
}

#[test]
fn test_bind() {
    let initial = CausalMonad::<i32, String>::pure(10);

    let next = CausalMonad::<i32, String>::bind(initial, |val| {
        let mut p = CausalMonad::<i32, String>::pure(val + 1);
        p.logs.add_entry("step1");
        p
    });

    if let EffectValue::Value(v) = next.value {
        assert_eq!(v, 11);
    } else {
        panic!("Expected Value(11)");
    }

    assert_eq!(next.logs.len(), 1);
}

#[test]
fn test_bind_error() {
    let mut initial = CausalMonad::<i32, String>::pure(10);
    // Inject error manually since pure doesn't allow it
    initial.error = Some(deep_causality_core::CausalityError::new(
        deep_causality_core::CausalityErrorEnum::InternalLogicError,
    ));

    let next =
        CausalMonad::<i32, String>::bind(initial, |val| CausalMonad::<i32, String>::pure(val + 1));

    assert!(next.error.is_some());
    if let EffectValue::Value(v) = next.value {
        assert_eq!(v, 0); // Default for i32
    }
}

#[test]
fn test_intervene() {
    let initial = CausalMonad::<i32, String>::pure(10);

    let intervened = CausalMonad::<i32, String>::intervene(initial, 99);

    if let EffectValue::Value(v) = intervened.value {
        assert_eq!(v, 99);
    } else {
        panic!("Expected Value(99)");
    }

    assert_eq!(intervened.logs.len(), 1); // "Intervention occurred"
}
