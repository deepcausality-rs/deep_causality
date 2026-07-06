/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::*;

#[test]
fn test_propagating_effect_intervene() {
    let effect = PropagatingEffect::pure(42);
    if let Some(v) = effect.value() {
        assert_eq!(*v, 42);
    } else {
        panic!("Expected Value(42)");
    }

    let intervened = effect.intervene(100);
    if let Some(v) = intervened.value() {
        assert_eq!(*v, 100);
    } else {
        panic!("Expected Value(100)");
    }
}

#[test]
fn test_propagating_effect_intervene_with_error() {
    let err = CausalityError::new(CausalityErrorEnum::Custom("Something bad happened".into()));
    let effect = PropagatingEffect::<i32>::from_error(err);

    assert!(effect.is_err());

    let intervened = effect.intervene(100);

    // Intervention should NOT happen if there is an error
    assert!(intervened.is_err());

    // An errored carrier has no value channel: value() lends nothing.
    assert!(intervened.value().is_none());
}

#[test]
fn test_propagating_process_intervene() {
    let process = PropagatingProcess::pure(10);
    let process = PropagatingProcess::with_state(process, "initial_state", None::<()>);

    if let Some(v) = process.value() {
        assert_eq!(*v, 10);
    }
    assert_eq!(*process.state(), "initial_state");

    let intervened = process.intervene(999);

    if let Some(v) = intervened.value() {
        assert_eq!(*v, 999);
    }
    assert_eq!(*intervened.state(), "initial_state"); // State should be preserved
}
