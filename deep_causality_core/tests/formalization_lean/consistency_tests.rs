/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witness for the functor-agreement law (`core.witness.agree`).
//!
//! Mirrors `lean/DeepCausalityFormal/Core/Consistency.lean`. The HKT process witness
//! `CausalEffectPropagationProcessWitness::fmap` and the inherent `CausalFlow::map` must produce the
//! same result on every carrier — value (`Some`), absence (`None`), error (`Err`), AND command
//! (`RelayTo`). Both now delegate to the total `CausalEffect::map`, so a command is **preserved**
//! (its leaf mapped) rather than collapsed. Deviation D15 is retired: the former arity-5 `.expect`
//! panic and the four-way `fmap` divergence (one erroring, one Noneing a command) are gone — checked
//! with NO `Ok(Value _)`-only restriction and no panic path.

use deep_causality_core::{
    CausalEffect, CausalEffectPropagationProcess, CausalEffectPropagationProcessWitness,
    CausalFlow, CausalityError, CausalityErrorEnum, EffectLog,
};
use deep_causality_haft::{Applicative, Functor};

type Witness = CausalEffectPropagationProcessWitness<(), (), CausalityError, EffectLog>;
type FnI64 = fn(i64) -> i64;

/// A stateless carrier with the given outcome.
fn carrier(
    outcome: Result<CausalEffect<i64>, CausalityError>,
) -> CausalEffectPropagationProcess<i64, (), (), CausalityError, EffectLog> {
    CausalEffectPropagationProcess::new(outcome, (), None, EffectLog::new())
}

/// A stateless function-carrying operand for `apply`.
fn fn_carrier(
    outcome: Result<CausalEffect<FnI64>, CausalityError>,
) -> CausalEffectPropagationProcess<FnI64, (), (), CausalityError, EffectLog> {
    CausalEffectPropagationProcess::new(outcome, (), None, EffectLog::new())
}

// ---- core.witness.agree : witness fmap = inherent fmap on every carrier ------------------------

/// THEOREM_MAP: core.witness.agree
#[test]
fn test_witness_agrees_with_inherent_fmap() {
    let f = |x: i64| x + 1;

    // Value carrier: both map the leaf to `Some(3)`.
    let witnessed = CausalFlow::from(Witness::fmap(carrier(Ok(CausalEffect::value(2))), f))
        .finish()
        .ok();
    let inherent = CausalFlow::from(carrier(Ok(CausalEffect::value(2))))
        .map(f)
        .finish()
        .ok();
    assert_eq!(witnessed, inherent);
    assert_eq!(inherent, Some(3));
}

#[test]
fn test_witness_agrees_on_none() {
    let f = |x: i64| x + 1;

    // None carrier: both stay `Ok(None)` — not an error, but no value to finish with.
    let witnessed = CausalFlow::from(Witness::fmap(carrier(Ok(CausalEffect::none())), f));
    let inherent = CausalFlow::from(carrier(Ok(CausalEffect::none()))).map(f);

    assert_eq!(witnessed.is_err(), inherent.is_err());
    assert!(!inherent.is_err()); // `Ok(None)` is not the error channel …
    assert!(witnessed.finish().is_err()); // … but has no value (both agree).
}

#[test]
fn test_witness_agrees_on_error() {
    let f = |x: i64| x + 1;
    let err = || CausalityError::new(CausalityErrorEnum::ValueNotAvailable);

    // Error carrier: `f` is not invoked (left zero); both stay in the error channel.
    let witnessed = CausalFlow::from(Witness::fmap(carrier(Err(err())), f));
    let inherent = CausalFlow::from(carrier(Err(err()))).map(f);

    assert_eq!(witnessed.is_err(), inherent.is_err());
    assert!(witnessed.is_err());
}

#[test]
fn test_witness_agrees_on_command() {
    let f = |x: i64| x + 1;
    let cmd = || carrier(Ok(CausalEffect::relay_to(3, CausalEffect::value(5))));

    // Both PRESERVE the command (same target) — neither errors nor collapses it to None (D15 fix).
    let witnessed = Witness::fmap(cmd(), f);
    let inherent = CausalFlow::from(cmd()).map(f).into_process();

    assert_eq!(witnessed.command_target(), Some(3));
    assert_eq!(inherent.command_target(), Some(3));

    // The command's sub-program leaf is mapped identically: value(5) -> value(6).
    let w_leaf = witnessed
        .into_parts()
        .0
        .ok()
        .and_then(CausalEffect::into_command)
        .and_then(|(_, sub)| sub.into_value());
    let i_leaf = inherent
        .into_parts()
        .0
        .ok()
        .and_then(CausalEffect::into_command)
        .and_then(|(_, sub)| sub.into_value());
    assert_eq!(w_leaf, i_leaf);
    assert_eq!(w_leaf, Some(6));
}

// ---- Applicative `apply`: total over the success channel (Consistency.lean apply_* theorems) ----

/// THEOREM_MAP: core.witness.agree
///
/// Mirrors `apply_none_yields_none`: a value-less function operand yields absence (`none()`), never
/// an `InternalLogicError`.
#[test]
fn test_apply_none_operand_yields_none() {
    let f_none = fn_carrier(Ok(CausalEffect::none()));
    let a = carrier(Ok(CausalEffect::value(5)));

    let out = <Witness as Applicative<Witness>>::apply(f_none, a);

    assert!(out.is_ok(), "value-less operand must not error");
    assert!(out.value().is_none());
}

/// THEOREM_MAP: core.witness.agree
///
/// Mirrors `apply_command_yields_none`: a command operand yields absence, never an error (commands
/// are folded by `CausalEffect::fold`, not applied).
#[test]
fn test_apply_command_operand_yields_none() {
    let inc: fn(i64) -> i64 = |x| x + 1;
    let f_ok = fn_carrier(Ok(CausalEffect::value(inc)));
    let a_cmd = carrier(Ok(CausalEffect::relay_to(3, CausalEffect::value(5))));

    let out = <Witness as Applicative<Witness>>::apply(f_ok, a_cmd);

    assert!(out.is_ok(), "a command operand must not error");
    assert!(out.value().is_none());
}

/// Both operands carrying a value: the applicative computes it (the `Pure(Some _)` fragment).
#[test]
fn test_apply_values_computes() {
    let inc: fn(i64) -> i64 = |x| x + 1;
    let f_ok = fn_carrier(Ok(CausalEffect::value(inc)));
    let a = carrier(Ok(CausalEffect::value(5)));

    let out = <Witness as Applicative<Witness>>::apply(f_ok, a);

    assert_eq!(out.value(), Some(&6));
}
