/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the success-channel laws of `CausalEffect`.
//!
//! Mirrors `lean/DeepCausalityFormal/Core/CausalEffect.lean`. The value content is `Option<V>`,
//! whose functor laws are `haft.functor.laws` (proved in `Haft/Functor.lean`); this witness checks
//! (1) the honest `Maybe` projection `into_value` (`Pure(Some v) ↦ Some v`, `Pure(None)`/command
//! `↦ None`) — the `core.causal_effect.into_value` id — and (2) that the total `CausalEffect::map`
//! lifts the `Option` functor through the value leaves. The 18 behavioural unit tests in
//! `tests/types/causal_effect/` cover the rest of the API.

use deep_causality_core::CausalEffect;

// ---- core.causal_effect.into_value : the honest Maybe projection -------------------------------

/// THEOREM_MAP: core.causal_effect.into_value
#[test]
fn test_causal_effect_into_value() {
    // Pure(Some v) ↦ Some v
    assert_eq!(CausalEffect::value(7i64).into_value(), Some(7));
    // Pure(None) ↦ None
    assert_eq!(CausalEffect::<i64>::none().into_value(), None);
    // command (RelayTo) ↦ None — honest: a command carries no value (no payload-drop caveat).
    let command = CausalEffect::relay_to(2, CausalEffect::value(5i64));
    assert!(command.is_command());
    assert_eq!(command.into_value(), None);
}

// ---- value functor = the Option functor (haft.functor.laws) ------------------------------------

/// The value functor is `Option` (`haft.functor.laws`): `map` applies `f` to the `Some` leaf and
/// passes `None` through, and `map id = id` — no bespoke value-type functor, no panic.
#[test]
fn test_causal_effect_value_functor_is_option() {
    assert_eq!(
        CausalEffect::value(3i64).map(|x| x + 1).into_value(),
        Some(4)
    );
    assert_eq!(
        CausalEffect::<i64>::none().map(|x| x + 1).into_value(),
        None
    );
    // Identity law lifted through the leaf.
    assert_eq!(CausalEffect::value(9i64).map(|x| x).into_value(), Some(9));
    // Total & uniform on a command: maps the sub-program's leaves, stays a command (no panic).
    let mapped = CausalEffect::relay_to(1, CausalEffect::value(4i64)).map(|x| x + 10);
    assert!(mapped.is_command());
    assert_eq!(mapped.command_target(), Some(1));
}
