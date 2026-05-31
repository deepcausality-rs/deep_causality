/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::*;

#[test]
fn test_propagating_effect_alternate_value() {
    let effect = PropagatingEffect::pure(42);
    if let EffectValue::Value(v) = effect.value {
        assert_eq!(v, 42);
    } else {
        panic!("Expected Value(42)");
    }

    let alternated = effect.alternate_value(100);
    if let EffectValue::Value(v) = alternated.value {
        assert_eq!(v, 100);
    } else {
        panic!("Expected Value(100)");
    }
}

#[test]
fn test_propagating_effect_alternate_value_with_error() {
    let err = CausalityError::new(CausalityErrorEnum::Custom("Something bad happened".into()));
    let effect = PropagatingEffect::<i32>::from_error(err);

    assert!(effect.error.is_some());

    let alternated = effect.alternate_value(100);

    // Alternation should NOT happen if there is an error
    assert!(alternated.error.is_some());

    // Value remains whatever it was (None for error state)
    assert!(matches!(alternated.value, EffectValue::None));
}

#[test]
fn test_propagating_process_alternate_value() {
    let process = PropagatingProcess::pure(10);
    let process = PropagatingProcess::with_state(process, "initial_state", None::<()>);

    if let EffectValue::Value(v) = process.value {
        assert_eq!(v, 10);
    }
    assert_eq!(process.state, "initial_state");

    let alternated = process.alternate_value(999);

    if let EffectValue::Value(v) = alternated.value {
        assert_eq!(v, 999);
    }
    // State must be preserved across a value-channel alternation.
    assert_eq!(alternated.state, "initial_state");
}

#[test]
fn test_alternate_value_appends_log_marker() {
    let effect = PropagatingEffect::pure(1_i32);
    let alternated = effect.alternate_value(2);
    assert!(alternated.logs.to_string().contains("!!ValueAlternation!!"));
}
