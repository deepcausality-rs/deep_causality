/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// Kani bounded-model-checking witnesses for the Causal Monad laws proved in Lean.
//
// Lean source of truth: `lean/DeepCausalityFormal/Core/CausalMonad.lean`.
// Bound via `lean/THEOREM_MAP.md`.
//
// The whole file is gated on `#![cfg(kani)]`: under normal `cargo build` / `cargo test` it
// compiles to nothing (no runtime cost, no coverage impact, no Bazel wiring). It is exercised
// only by `cargo kani --tests -p deep_causality_core`, at which point the `kani` crate is
// injected by the Kani driver.
//
// Kani cannot quantify over the continuation `f` — the monad laws are higher-order and bounded
// model checkers are first-order (Formalization.md §3, L3). So each harness fixes one concrete,
// representative continuation and verifies the law over all carried values.
//
// NOTE on the former W-well-formedness harness obligation: with value-XOR-error encoded as ONE
// channel (`outcome: Result<EffectValue<T>, E>`, precondition P2), the invalid state
// "value AND error" is unrepresentable — there is nothing left to model-check. The obligation
// is discharged by construction.
#![cfg(kani)]

use deep_causality_core::{
    CausalMonad, CausalityError, CausalityErrorEnum, EffectLog, EffectValue, PropagatingProcess,
};
use deep_causality_haft::{LogAddEntry, LogSize};

/// Stateful carrier: `i32` value, `i32` Markovian state, `()` context.
type P = PropagatingProcess<i32, i32, ()>;

/// A fixed, representative continuation: increments both the carried value and the threaded
/// state, contributes an empty log, and preserves context. Mirrors the arbitrary `f` in the
/// Lean theorem `bind_left_id`.
fn cont(v: EffectValue<i32>, state: i32, ctx: Option<()>) -> P {
    let val = v.into_value().unwrap_or_default();
    P::new(
        Ok(EffectValue::Value(val.wrapping_add(1))),
        state.wrapping_add(1),
        ctx,
        EffectLog::new(),
    )
}

/// A nondeterministic carrier covering every representable shape: errored, value-less, or
/// carrying an arbitrary value — with arbitrary state and a log of bounded length.
fn any_process() -> P {
    let state: i32 = kani::any();
    let outcome: Result<EffectValue<i32>, CausalityError> = match kani::any::<u8>() % 3 {
        0 => Err(CausalityError::new(CausalityErrorEnum::ValueNotAvailable)),
        1 => Ok(EffectValue::None),
        _ => Ok(EffectValue::Value(kani::any())),
    };
    let mut logs = EffectLog::new();
    if kani::any() {
        logs.add_entry("step");
    }
    P::new(outcome, state, None, logs)
}

/// Witness for `THEOREM_MAP: core.causal_monad.left_id`.
///
/// Left identity `pure(a) >>= f = f a`: since `pure` injects the value with the default state,
/// no context, and an empty log, binding the continuation must equal applying it at exactly
/// that injected point. Verified for all `a: i32`.
#[kani::proof]
fn causal_monad_left_identity() {
    let a: i32 = kani::any();

    // LHS: pure(a) >>= cont
    let lhs = <P as CausalMonad>::pure(a).bind(cont);

    // RHS: cont applied where `pure` injects — value = Value(a), state = default, context = None.
    let rhs = cont(EffectValue::Value(a), i32::default(), None);

    assert!(lhs == rhs);
}

/// Witness for `THEOREM_MAP: core.causal_monad.right_id`.
///
/// Right identity `m >>= eta = m`, UNCONDITIONALLY — including errored and value-less
/// carriers. `eta` is the Kleisli unit of the state-threading monad: it re-emits the received
/// value, state, and context with an empty log. This is the law P2 unblocked: no representable
/// carrier state exists on which it could fail.
#[kani::proof]
fn causal_monad_right_identity() {
    let m = any_process();
    let expected = m.clone();

    let result = m.bind(|v, s, c| P::new(Ok(v), s, c, EffectLog::new()));

    assert!(result == expected);
}

/// Witness for `THEOREM_MAP: core.causal_monad.assoc` (bounded, fixed continuations).
///
/// Associativity `(m >>= f) >>= g = m >>= (|x| f(x) >>= g)` over every representable carrier
/// shape, with two distinct representative continuations.
#[kani::proof]
fn causal_monad_associativity() {
    fn g(v: EffectValue<i32>, state: i32, ctx: Option<()>) -> P {
        let val = v.into_value().unwrap_or_default();
        let mut logs = EffectLog::new();
        logs.add_entry("g");
        P::new(
            Ok(EffectValue::Value(val.wrapping_mul(2))),
            state.wrapping_sub(1),
            ctx,
            logs,
        )
    }

    let m = any_process();
    let m2 = m.clone();

    let lhs = m.bind(cont).bind(g);
    let rhs = m2.bind(|v, s, c| cont(v, s, c).bind(g));

    assert!(lhs == rhs);
}

/// Raise is a left zero: on an errored carrier, `bind` must NOT invoke the continuation, and
/// error, state, context, and logs survive verbatim (Lean: `bind_raise_left_zero`).
#[kani::proof]
fn causal_monad_short_circuit() {
    let state: i32 = kani::any();
    let mut logs = EffectLog::new();
    logs.add_entry("before");
    let errored = P::new(
        Err(CausalityError::new(CausalityErrorEnum::ValueNotAvailable)),
        state,
        None,
        logs,
    );
    let expected = errored.clone();

    let mut ran = false;
    let result = errored.bind(|v, s, c| {
        ran = true;
        cont(v, s, c)
    });

    assert!(!ran);
    assert!(result == expected);
}

/// Log monotonicity: `bind` never loses audit history — the output log is at least as long
/// as the input log (the continuation may only append).
#[kani::proof]
fn causal_monad_log_monotone() {
    let m = any_process();
    let len_before = m.logs().len();

    let result = m.bind(cont);

    assert!(result.logs().len() >= len_before);
}
