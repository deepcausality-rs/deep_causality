/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::*;

#[derive(Clone, Debug, Default, PartialEq)]
struct St {
    counter: u32,
}

#[test]
fn test_propagating_process_alternate_state_replaces_only_state() {
    let initial = PropagatingProcess::pure(42_i32);
    let process = PropagatingProcess::with_state(initial, St { counter: 3 }, None::<()>);

    assert_eq!(process.state.counter, 3);

    let new_state = St { counter: 99 };
    let alternated = process.alternate_state(new_state.clone());

    // State must change.
    assert_eq!(alternated.state, new_state);
    // Value must be preserved.
    if let EffectValue::Value(v) = alternated.value {
        assert_eq!(v, 42);
    } else {
        panic!("Expected Value(42)");
    }
    // Context must be preserved (None here).
    assert!(alternated.context.is_none());
    // No error must be introduced.
    assert!(alternated.error.is_none());
}

#[test]
fn test_alternate_state_with_error_is_noop() {
    let err = CausalityError::new(CausalityErrorEnum::Custom("upstream failure".into()));
    let process = PropagatingProcess::<i32, St, ()>::from_error(err);
    let original_state = process.state.clone();

    let alternated = process.alternate_state(St { counter: 99 });

    // Error must propagate, state must not change.
    assert!(alternated.error.is_some());
    assert_eq!(alternated.state, original_state);
}

#[test]
fn test_alternate_state_appends_log_marker() {
    let initial = PropagatingProcess::pure(1_i32);
    let process = PropagatingProcess::with_state(initial, St::default(), None::<()>);
    let alternated = process.alternate_state(St { counter: 1 });
    assert!(alternated.logs.to_string().contains("!!StateAlternation!!"));
}

#[test]
fn test_alternate_state_on_propagating_effect_is_unit_only() {
    // Documented behaviour: PropagatingEffect (State = ()) accepts the
    // call but only the audit log changes.
    let effect = PropagatingEffect::pure(7_i32);
    let alternated = effect.alternate_state(());
    if let EffectValue::Value(v) = alternated.value {
        assert_eq!(v, 7);
    }
    assert!(alternated.logs.to_string().contains("!!StateAlternation!!"));
}
