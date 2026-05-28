/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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

    // The closure must not run when the upstream process already carries an error.
    let mut called = false;
    let next = CausalMonad::<i32, String>::bind(initial, |val| {
        called = true;
        CausalMonad::<i32, String>::pure(val + 1)
    });

    assert!(!called, "bind must short-circuit on error and not call f");
    assert!(next.error.is_some(), "the upstream error is preserved");
    // An errored chain carries NO value. It must be EffectValue::None, never a
    // fabricated Value(default).
    assert!(
        matches!(next.value, EffectValue::None),
        "errored bind must yield EffectValue::None, not a fabricated default value"
    );
}

#[test]
fn test_bind_error_preserves_state_context_and_logs() {
    let mut initial = CausalMonad::<i32, String>::pure(10);
    initial.state = 7;
    initial.context = Some("ctx".to_string());
    initial.logs.add_entry("upstream");
    initial.error = Some(deep_causality_core::CausalityError::new(
        deep_causality_core::CausalityErrorEnum::InternalLogicError,
    ));

    let next =
        CausalMonad::<i32, String>::bind(initial, |val| CausalMonad::<i32, String>::pure(val + 1));

    assert!(matches!(next.value, EffectValue::None));
    assert_eq!(
        next.state, 7,
        "state is carried across the error short-circuit"
    );
    assert_eq!(next.context, Some("ctx".to_string()));
    assert_eq!(next.logs.len(), 1, "upstream log is preserved on error");
}

#[test]
fn test_bind_none_value_surfaces_error_and_keeps_none() {
    // A process with no error but EffectValue::None is an internal-logic
    // inconsistency; bind must surface an error and keep the value None.
    let mut initial = CausalMonad::<i32, String>::pure(10);
    initial.value = EffectValue::None;

    let mut called = false;
    let next = CausalMonad::<i32, String>::bind(initial, |val| {
        called = true;
        CausalMonad::<i32, String>::pure(val + 1)
    });

    assert!(
        !called,
        "bind must not call f when there is no value to pass"
    );
    assert!(next.error.is_some(), "a missing value surfaces an error");
    assert!(matches!(next.value, EffectValue::None));
}

#[test]
fn test_intervene() {
    let initial = CausalMonad::<i32, String>::pure(10);

    let intervened = initial.intervene(99);

    if let EffectValue::Value(v) = intervened.value {
        assert_eq!(v, 99);
    } else {
        panic!("Expected Value(99)");
    }

    assert_eq!(intervened.logs.len(), 1); // "Intervention occurred"
}
