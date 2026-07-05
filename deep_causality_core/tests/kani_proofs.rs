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
#![cfg(kani)]

use deep_causality_core::{
    CausalEffectPropagationProcess, CausalMonad, EffectLog, EffectValue, PropagatingProcess,
};

/// Stateful carrier: `i32` value, `i32` Markovian state, `()` context.
type P = PropagatingProcess<i32, i32, ()>;

/// A fixed, representative continuation: increments both the carried value and the threaded
/// state, contributes an empty log, and preserves context. Mirrors the arbitrary `f` in the
/// Lean theorem `bind_left_id`.
fn cont(v: EffectValue<i32>, state: i32, ctx: Option<()>) -> P {
    let val = v.into_value().unwrap_or_default();
    CausalEffectPropagationProcess {
        value: EffectValue::Value(val.wrapping_add(1)),
        state: state.wrapping_add(1),
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    }
}

/// Witness for `THEOREM_MAP: core.causal_monad.left_id`.
///
/// Left identity `pure(a) >>= f = f a`: since `pure` injects the value with the default state,
/// no context, no error and an empty log, binding the continuation must equal applying it at
/// exactly that injected point. Verified for all `a: i32`.
#[kani::proof]
fn causal_monad_left_identity() {
    let a: i32 = kani::any();

    // LHS: pure(a) >>= cont
    let lhs = <P as CausalMonad>::pure(a).bind(cont);

    // RHS: cont applied where `pure` injects — value = Value(a), state = default, context = None.
    let rhs = cont(EffectValue::Value(a), i32::default(), None);

    assert!(lhs == rhs);
}
