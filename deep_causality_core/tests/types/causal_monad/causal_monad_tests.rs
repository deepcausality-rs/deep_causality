/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{
    CausalEffectPropagationProcess, CausalMonad, CausalityError, CausalityErrorEnum, EffectLog,
    EffectValue, Intervenable, PropagatingProcess,
};
use deep_causality_haft::{LogAddEntry, LogSize};

/// Stateful carrier used across these tests: `i32` value, `i32` Markovian state,
/// `String` context. `PropagatingProcess<T, S, C>` implements the `CausalMonad`
/// trait; `pure` / `bind` are also available as inherent methods.
type P<T> = PropagatingProcess<T, i32, String>;

#[test]
fn test_pure() {
    let process: P<i32> = PropagatingProcess::pure(42);

    assert!(matches!(process.value, EffectValue::Value(42)));
    assert_eq!(process.state, 0); // Default for i32
    assert!(process.context.is_none());
    assert!(process.error.is_none());
    assert!(process.logs.is_empty());
}

#[test]
fn test_pure_via_trait() {
    // The trait `pure` and the inherent `pure` produce the same process.
    let via_trait: P<i32> = <P<i32> as CausalMonad>::pure(7);
    let via_inherent: P<i32> = PropagatingProcess::pure(7);
    assert_eq!(via_trait, via_inherent);
}

#[test]
fn test_bind() {
    let initial: P<i32> = PropagatingProcess::pure(10);

    let next = initial.bind(|v, state, ctx| {
        let val = v.into_value().unwrap_or_default();
        let mut p: P<i32> = CausalEffectPropagationProcess {
            value: EffectValue::Value(val + 1),
            state,
            context: ctx,
            error: None,
            logs: EffectLog::new(),
        };
        p.logs.add_entry("step1");
        p
    });

    assert!(matches!(next.value, EffectValue::Value(11)));
    assert_eq!(next.logs.len(), 1);
}

/// A step that increments both the value and the Markovian state.
fn inc_step(v: EffectValue<i32>, state: i32, ctx: Option<String>) -> P<i32> {
    let val = v.into_value().unwrap_or_default();
    CausalEffectPropagationProcess {
        value: EffectValue::Value(val + 1),
        state: state + 10,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    }
}

#[test]
fn test_bind_threads_and_updates_state() {
    // The defining property of the corrected monad: `bind` threads the state to
    // the continuation AND carries the continuation's updated state forward.
    // The removed value-only witness binds froze the state at its initial value;
    // this test locks in that the state evolves across the chain.
    let p0: P<i32> = PropagatingProcess::pure(0);
    assert_eq!(p0.state, 0);

    let p2 = p0.bind(inc_step).bind(inc_step);

    assert!(matches!(p2.value, EffectValue::Value(2)));
    assert_eq!(
        p2.state, 20,
        "state must thread and update across binds, not freeze at the initial value"
    );
}

#[test]
fn test_bind_error() {
    let mut initial: P<i32> = PropagatingProcess::pure(10);
    // Inject an error manually since `pure` cannot.
    initial.error = Some(CausalityError::new(CausalityErrorEnum::InternalLogicError));

    // The continuation must not run when the upstream process already errored.
    let mut called = false;
    let next = initial.bind(|v, state, ctx| {
        called = true;
        let val = v.into_value().unwrap_or_default();
        CausalEffectPropagationProcess {
            value: EffectValue::Value(val + 1),
            state,
            context: ctx,
            error: None,
            logs: EffectLog::new(),
        }
    });

    assert!(!called, "bind must short-circuit on error and not call f");
    assert!(next.error.is_some(), "the upstream error is preserved");
    // An errored chain carries NO value: EffectValue::None, never a fabricated default.
    assert!(
        matches!(next.value, EffectValue::None),
        "errored bind must yield EffectValue::None"
    );
}

#[test]
fn test_bind_error_preserves_state_context_and_logs() {
    let mut initial: P<i32> = PropagatingProcess::pure(10);
    initial.state = 7;
    initial.context = Some("ctx".to_string());
    initial.logs.add_entry("upstream");
    initial.error = Some(CausalityError::new(CausalityErrorEnum::InternalLogicError));

    let next = initial.bind(inc_step);

    assert!(matches!(next.value, EffectValue::None));
    assert_eq!(
        next.state, 7,
        "state is carried across the error short-circuit"
    );
    assert_eq!(next.context, Some("ctx".to_string()));
    assert_eq!(next.logs.len(), 1, "upstream log is preserved on error");
}

#[test]
fn test_intervene() {
    let initial: P<i32> = PropagatingProcess::pure(10);

    let intervened = initial.intervene(99);

    assert!(matches!(intervened.value, EffectValue::Value(99)));
    assert_eq!(intervened.logs.len(), 1); // "Intervention occurred"
}

#[test]
fn test_fmap_maps_value_and_preserves_state_context_logs() {
    let mut initial: P<i32> = PropagatingProcess::pure(10);
    initial.state = 7;
    initial.context = Some("ctx".to_string());
    initial.logs.add_entry("upstream");

    let mapped = initial.fmap(|x| x * 2);

    assert!(matches!(mapped.value, EffectValue::Value(20)));
    assert_eq!(
        mapped.state, 7,
        "fmap preserves state, it does not thread it"
    );
    assert_eq!(mapped.context, Some("ctx".to_string()));
    assert!(mapped.error.is_none());
    assert_eq!(mapped.logs.len(), 1, "fmap preserves the upstream log");
}

#[test]
fn test_fmap_is_type_changing() {
    // fmap may change the value type; state and context types stay fixed.
    let initial: P<i32> = PropagatingProcess::pure(42);

    let mapped: P<String> = initial.fmap(|x| x.to_string());

    assert!(matches!(mapped.value, EffectValue::Value(ref s) if s == "42"));
}

#[test]
fn test_fmap_short_circuits_on_error_without_calling_f() {
    let mut initial: P<i32> = PropagatingProcess::pure(10);
    initial.state = 7;
    initial.logs.add_entry("upstream");
    initial.error = Some(CausalityError::new(CausalityErrorEnum::InternalLogicError));

    let mut called = false;
    let mapped = initial.fmap(|x| {
        called = true;
        x * 2
    });

    assert!(!called, "fmap must short-circuit on error and not call f");
    assert!(mapped.error.is_some(), "the upstream error is preserved");
    assert!(
        matches!(mapped.value, EffectValue::None),
        "an errored fmap yields EffectValue::None"
    );
    assert_eq!(mapped.state, 7, "state is carried across the short-circuit");
    assert_eq!(mapped.logs.len(), 1, "upstream log is preserved on error");
}

#[test]
fn test_fmap_passes_none_through() {
    let mut initial: P<i32> = PropagatingProcess::pure(0);
    initial.value = EffectValue::None;

    let mut called = false;
    let mapped = initial.fmap(|x| {
        called = true;
        x * 2
    });

    assert!(!called, "there is no value to map, so f must not run");
    assert!(matches!(mapped.value, EffectValue::None));
    assert!(mapped.error.is_none(), "a None carrier is not an error");
}
