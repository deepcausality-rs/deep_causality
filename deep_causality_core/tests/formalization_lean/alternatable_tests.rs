/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the lens-family laws (`core.alternatable.*`).
//!
//! Mirrors `lean/DeepCausalityFormal/Core/Alternatable.lean`. The `alternate_*` setters are lenses
//! on the (value | state | context) channels up-to-log: every successful set appends one audit
//! entry (D9), so the lens laws hold on the log-erasing projection and set-set grows the log. Every
//! setter is a no-op on an errored carrier. One `#[test]` per THEOREM_MAP id.

use deep_causality_core::{
    AlternatableContext, AlternatableState, AlternatableValue, CausalEffect,
    CausalEffectPropagationProcess, CausalityError, CausalityErrorEnum, EffectLog,
    PropagatingProcess,
};
use deep_causality_haft::LogSize;

type P<T> = PropagatingProcess<T, i32, String>;

fn with_channels(value: i32, state: i32, ctx: &str) -> P<i32> {
    CausalEffectPropagationProcess::new(
        Ok(CausalEffect::value(value)),
        state,
        Some(ctx.to_string()),
        EffectLog::new(),
    )
}

// ---- core.alternatable.set_get -----------------------------------------------------------------

/// THEOREM_MAP: core.alternatable.set_get
#[test]
fn test_alternatable_set_get() {
    // Set each channel; read back exactly what was set.
    assert!(matches!(P::pure(10).alternate_value(99).value(), Some(&99)));
    assert_eq!(*P::pure(10).alternate_state(42).state(), 42);
    assert_eq!(
        P::pure(10).alternate_context("ctx".to_string()).context(),
        &Some("ctx".to_string())
    );
}

// ---- core.alternatable.set_set_proj (up-to-log) ------------------------------------------------

/// THEOREM_MAP: core.alternatable.set_set_proj
#[test]
fn test_alternatable_set_set_up_to_log() {
    // Projected (value) idempotence: the second write wins.
    let twice = P::pure(0).alternate_value(1).alternate_value(2);
    let once = P::pure(0).alternate_value(2);
    assert!(matches!(twice.value(), Some(&2)));
    assert!(matches!(once.value(), Some(&2)));

    // …but NOT on the nose: the full carrier grows the log by one entry per write (D9).
    assert_eq!(twice.logs().len(), 2);
    assert_eq!(once.logs().len(), 1);
}

// ---- core.alternatable.channel_independence ----------------------------------------------------

/// THEOREM_MAP: core.alternatable.channel_independence
#[test]
fn test_alternatable_channel_independence() {
    // alternate_value leaves state and context untouched.
    let v = with_channels(1, 5, "c").alternate_value(9);
    assert!(matches!(v.value(), Some(&9)));
    assert_eq!(*v.state(), 5);
    assert_eq!(v.context(), &Some("c".to_string()));

    // alternate_state leaves value and context untouched.
    let s = with_channels(1, 5, "c").alternate_state(7);
    assert!(matches!(s.value(), Some(&1)));
    assert_eq!(*s.state(), 7);
    assert_eq!(s.context(), &Some("c".to_string()));

    // alternate_context leaves value and state untouched.
    let c = with_channels(1, 5, "c").alternate_context("d".to_string());
    assert!(matches!(c.value(), Some(&1)));
    assert_eq!(*c.state(), 5);
    assert_eq!(c.context(), &Some("d".to_string()));

    // clear_context is the None-setting counterpart.
    let cleared = with_channels(1, 5, "c").clear_context();
    assert_eq!(cleared.context(), &None);
    assert!(matches!(cleared.value(), Some(&1)));
}

// ---- core.alternatable.error_noop --------------------------------------------------------------

/// THEOREM_MAP: core.alternatable.error_noop
#[test]
fn test_alternatable_error_noop() {
    let errored = || -> P<i32> {
        PropagatingProcess::from_error(CausalityError::new(CausalityErrorEnum::InternalLogicError))
    };

    // Every setter (and clear_context) is a no-op on an errored carrier.
    assert!(errored().alternate_value(9).error().is_some());
    assert!(errored().alternate_value(9).value().is_none());
    assert!(errored().alternate_state(9).error().is_some());
    assert!(
        errored()
            .alternate_context("x".to_string())
            .error()
            .is_some()
    );
    assert!(errored().clear_context().error().is_some());
    // No audit entry is appended on the errored no-op path.
    assert_eq!(errored().alternate_value(9).logs().len(), 0);
}
