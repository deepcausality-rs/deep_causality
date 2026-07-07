/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{
    AlternatableValue, CausalEffect, CausalEffectPropagationProcess, CausalMonad, CausalityError,
    CausalityErrorEnum, EffectLog, PropagatingProcess,
};
use deep_causality_haft::{LogAddEntry, LogSize};

/// Stateful carrier used across these tests: `i32` value, `i32` Markovian state,
/// `String` context. `PropagatingProcess<T, S, C>` implements the `CausalMonad`
/// trait; `pure` / `bind` are also available as inherent methods.
type P<T> = PropagatingProcess<T, i32, String>;

#[test]
fn test_pure() {
    let process: P<i32> = PropagatingProcess::pure(42);

    assert!(matches!(process.value(), Some(&42)));
    assert_eq!(*process.state(), 0); // Default for i32
    assert!(process.context().is_none());
    assert!(process.error().is_none());
    assert!(process.logs().is_empty());
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
        let mut logs = EffectLog::new();
        logs.add_entry("step1");
        CausalEffectPropagationProcess::new(Ok(CausalEffect::value(val + 1)), state, ctx, logs)
    });

    assert!(matches!(next.value(), Some(&11)));
    assert_eq!(next.logs().len(), 1);
}

/// A step that increments both the value and the Markovian state.
fn inc_step(v: CausalEffect<i32>, state: i32, ctx: Option<String>) -> P<i32> {
    let val = v.into_value().unwrap_or_default();
    CausalEffectPropagationProcess::new(
        Ok(CausalEffect::value(val + 1)),
        state + 10,
        ctx,
        EffectLog::new(),
    )
}

#[test]
fn test_bind_threads_and_updates_state() {
    // The defining property of the corrected monad: `bind` threads the state to
    // the continuation AND carries the continuation's updated state forward.
    // The removed value-only witness binds froze the state at its initial value;
    // this test locks in that the state evolves across the chain.
    let p0: P<i32> = PropagatingProcess::pure(0);
    assert_eq!(*p0.state(), 0);

    let p2 = p0.bind(inc_step).bind(inc_step);

    assert!(matches!(p2.value(), Some(&2)));
    assert_eq!(
        *p2.state(),
        20,
        "state must thread and update across binds, not freeze at the initial value"
    );
}

#[test]
fn test_bind_error() {
    // Value and error are one channel: an errored carrier is constructed as `Err`
    // and cannot also hold a value.
    let initial: P<i32> =
        PropagatingProcess::from_error(CausalityError::new(CausalityErrorEnum::InternalLogicError));

    // The continuation must not run when the upstream process already errored.
    let mut called = false;
    let next = initial.bind(|v, state, ctx| {
        called = true;
        let val = v.into_value().unwrap_or_default();
        CausalEffectPropagationProcess::new(
            Ok(CausalEffect::value(val + 1)),
            state,
            ctx,
            EffectLog::new(),
        )
    });

    assert!(!called, "bind must short-circuit on error and not call f");
    assert!(next.error().is_some(), "the upstream error is preserved");
    // An errored chain carries NO value: value and error are one channel.
    assert!(
        next.value().is_none(),
        "an errored bind must yield no value"
    );
}

#[test]
fn test_bind_error_preserves_state_context_and_logs() {
    let mut logs = EffectLog::new();
    logs.add_entry("upstream");
    let initial: P<i32> = CausalEffectPropagationProcess::new(
        Err(CausalityError::new(CausalityErrorEnum::InternalLogicError)),
        7,
        Some("ctx".to_string()),
        logs,
    );

    let next = initial.bind(inc_step);

    assert!(next.is_err());
    assert!(next.value().is_none());
    assert_eq!(
        *next.state(),
        7,
        "state is carried across the error short-circuit"
    );
    assert_eq!(next.context(), &Some("ctx".to_string()));
    assert_eq!(next.logs().len(), 1, "upstream log is preserved on error");
}

#[test]
fn test_alternate_value() {
    let initial: P<i32> = PropagatingProcess::pure(10);

    let alternated = initial.alternate_value(99);

    assert!(matches!(alternated.value(), Some(&99)));
    assert_eq!(alternated.logs().len(), 1); // value alternation occurred
}

#[test]
fn test_fmap_maps_value_and_preserves_state_context_logs() {
    let mut logs = EffectLog::new();
    logs.add_entry("upstream");
    let initial: P<i32> = CausalEffectPropagationProcess::new(
        Ok(CausalEffect::value(10)),
        7,
        Some("ctx".to_string()),
        logs,
    );

    let mapped = initial.fmap(|x| x * 2);

    assert!(matches!(mapped.value(), Some(&20)));
    assert_eq!(
        *mapped.state(),
        7,
        "fmap preserves state, it does not thread it"
    );
    assert_eq!(mapped.context(), &Some("ctx".to_string()));
    assert!(mapped.error().is_none());
    assert_eq!(mapped.logs().len(), 1, "fmap preserves the upstream log");
}

#[test]
fn test_fmap_is_type_changing() {
    // fmap may change the value type; state and context types stay fixed.
    let initial: P<i32> = PropagatingProcess::pure(42);

    let mapped: P<String> = initial.fmap(|x| x.to_string());

    assert!(matches!(mapped.value(), Some(s) if s == "42"));
}

#[test]
fn test_fmap_short_circuits_on_error_without_calling_f() {
    let mut logs = EffectLog::new();
    logs.add_entry("upstream");
    let initial: P<i32> = CausalEffectPropagationProcess::new(
        Err(CausalityError::new(CausalityErrorEnum::InternalLogicError)),
        7,
        None,
        logs,
    );

    let mut called = false;
    let mapped = initial.fmap(|x| {
        called = true;
        x * 2
    });

    assert!(!called, "fmap must short-circuit on error and not call f");
    assert!(mapped.error().is_some(), "the upstream error is preserved");
    assert!(mapped.value().is_none(), "an errored fmap yields no value");
    assert_eq!(
        *mapped.state(),
        7,
        "state is carried across the short-circuit"
    );
    assert_eq!(mapped.logs().len(), 1, "upstream log is preserved on error");
}

#[test]
fn test_fmap_passes_none_through() {
    let initial: P<i32> = PropagatingProcess::none();

    let mut called = false;
    let mapped = initial.fmap(|x| {
        called = true;
        x * 2
    });

    assert!(!called, "there is no value to map, so f must not run");
    assert!(mapped.effect().is_some_and(CausalEffect::is_none));
    assert!(mapped.error().is_none(), "a None carrier is not an error");
}

/// Witness for `THEOREM_MAP: core.causal_monad.right_id`.
///
/// Right identity holds UNCONDITIONALLY: `bind(m, eta) == m` for every carrier — including
/// errored ones, on which `bind` returns the carrier verbatim (P2: value and error are one
/// channel, so no carrier state exists on which the law could fail). `eta` re-emits the
/// received value, state, and context with an empty log — the Kleisli unit of the
/// state-threading monad. Lean proof: `Core/CausalMonad.lean :: bind_right_id`.
#[test]
fn test_right_identity_unconditional() {
    let mut logs = EffectLog::new();
    logs.add_entry("history");

    let carriers: Vec<P<i32>> = vec![
        PropagatingProcess::pure(42),
        PropagatingProcess::none(),
        PropagatingProcess::new(
            Ok(CausalEffect::value(7)),
            3,
            Some("ctx".to_string()),
            logs.clone(),
        ),
        // The errored carrier — the case that failed before P2.
        PropagatingProcess::new(
            Err(CausalityError::new(CausalityErrorEnum::ValueNotAvailable)),
            9,
            Some("ctx".to_string()),
            logs,
        ),
    ];

    for m in carriers {
        let expected = m.clone();
        let result = m.bind(|v, s, c| PropagatingProcess::new(Ok(v), s, c, EffectLog::new()));
        assert_eq!(result, expected, "bind(m, eta) must equal m verbatim");
    }
}

/// Witness for `THEOREM_MAP: core.causal_monad.assoc`.
///
/// Associativity `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`, exercised across an
/// erroring continuation (the spec scenario): `f` raises, `g` must never run on either side,
/// and both sides agree — including logs (the Writer monoid law).
/// Lean proof: `Core/CausalMonad.lean :: bind_assoc`.
#[test]
fn test_associativity_across_erroring_continuation() {
    let erroring_f = |_v: CausalEffect<i32>, s: i32, c: Option<String>| -> P<i32> {
        let mut logs = EffectLog::new();
        logs.add_entry("f raised");
        PropagatingProcess::new(
            Err(CausalityError::new(CausalityErrorEnum::ValueNotAvailable)),
            s,
            c,
            logs,
        )
    };
    let g = |v: CausalEffect<i32>, s: i32, c: Option<String>| -> P<i32> {
        let val = v.into_value().unwrap_or_default();
        let mut logs = EffectLog::new();
        logs.add_entry("g ran");
        PropagatingProcess::new(Ok(CausalEffect::value(val * 10)), s + 1, c, logs)
    };

    let mut in_logs = EffectLog::new();
    in_logs.add_entry("start");
    let m: P<i32> = PropagatingProcess::new(Ok(CausalEffect::value(5)), 1, None, in_logs);
    let m2 = m.clone();

    let lhs = m.bind(erroring_f).bind(g);
    let rhs = m2.bind(|v, s, c| erroring_f(v, s, c).bind(g));

    assert_eq!(lhs, rhs, "associativity must hold across an erroring f");
    assert!(lhs.is_err(), "the raise propagates");
    assert_eq!(lhs.logs().len(), 2, "logs: start + f raised; g never ran");

    // And the plain (non-erroring) chain agrees too.
    let ok_f = |v: CausalEffect<i32>, s: i32, c: Option<String>| -> P<i32> {
        let val = v.into_value().unwrap_or_default();
        PropagatingProcess::new(Ok(CausalEffect::value(val + 1)), s, c, EffectLog::new())
    };
    let m3: P<i32> = PropagatingProcess::pure(5);
    let m4 = m3.clone();
    assert_eq!(
        m3.bind(ok_f).bind(g),
        m4.bind(|v, s, c| ok_f(v, s, c).bind(g))
    );
}
