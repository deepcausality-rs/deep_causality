/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Consistency tests between the two HKT witnesses that project to the
//! same `CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>`
//! carrier:
//!
//! - `PropagatingEffectWitness<CausalityError, EffectLog>` (from
//!   `propagating_effect/hkt.rs`)
//! - `PropagatingProcessWitness<(), ()>` (from `propagating_process/hkt.rs`)
//!
//! Both witnesses ship their own independently-written `Functor`,
//! `Applicative`, and `Monad` impls. Since they project to the same
//! concrete type, their methods must produce identical results on the
//! shared carrier. A future refactor of one impl must not silently diverge
//! from the other; these tests pin that consistency.
//!
//! No iso wrapper is used. The carrier is literally the same type
//! (`PropagatingEffect<T>` and `PropagatingProcess<T, (), ()>` are type
//! aliases for the same `CausalEffectPropagationProcess<...>`), so any
//! `NaturalIso` between the two witnesses would have identity bodies and
//! add no information. The honest framing is "two dispatch paths into the
//! same concrete operations should agree."
//!
//! See `openspec/changes/implement-isomorphism/design.md` D7 for the
//! rationale.

use deep_causality_core::{
    CausalEffect, CausalityError, EffectLog, PropagatingEffect, PropagatingEffectWitness,
    PropagatingProcess, PropagatingProcessWitness,
};
use deep_causality_haft::{Functor, Pure};

type EffectW = PropagatingEffectWitness<CausalityError, EffectLog>;
type ProcessW = PropagatingProcessWitness<(), ()>;

/// `fmap` consistency on a same-type closure (`i32 -> i32`).
#[test]
fn fmap_same_type_agrees_across_witnesses() {
    let val: PropagatingEffect<i32> = EffectW::pure(42);
    let proc: PropagatingProcess<i32, (), ()> = val.clone();

    let via_effect = <EffectW as Functor<EffectW>>::fmap(val, |x| x * 2);
    let via_process = <ProcessW as Functor<ProcessW>>::fmap(proc, |x| x * 2);

    assert_eq!(
        via_effect, via_process,
        "fmap diverges between PropagatingEffectWitness and PropagatingProcessWitness \
         on the shared carrier (same-type closure)"
    );
}

/// `fmap` consistency on a type-changing closure (`i32 -> bool`). The two
/// impls take different code paths for the same operation; they must still
/// produce the same output on the shared carrier.
#[test]
fn fmap_type_changing_agrees_across_witnesses() {
    let val: PropagatingEffect<i32> = EffectW::pure(7);
    let proc: PropagatingProcess<i32, (), ()> = val.clone();

    let via_effect = <EffectW as Functor<EffectW>>::fmap(val, |x| x % 2 == 0);
    let via_process = <ProcessW as Functor<ProcessW>>::fmap(proc, |x| x % 2 == 0);

    assert_eq!(
        via_effect, via_process,
        "fmap diverges between PropagatingEffectWitness and PropagatingProcessWitness \
         on the shared carrier (type-changing closure)"
    );
}

/// `fmap` consistency on a `None` (absence) carrier — the case that used to diverge
/// (`InternalLogicError`). Both witnesses are now total: a `None` effect passes through unchanged.
#[test]
fn fmap_on_none_agrees_across_witnesses() {
    let val: PropagatingEffect<i32> = PropagatingEffect::new(Ok(CausalEffect::none()), (), None, EffectLog::new());
    let proc: PropagatingProcess<i32, (), ()> = val.clone();

    let via_effect = <EffectW as Functor<EffectW>>::fmap(val, |x| x * 2);
    let via_process = <ProcessW as Functor<ProcessW>>::fmap(proc, |x| x * 2);

    assert!(via_effect.is_ok() && via_effect.value().is_none());
    assert_eq!(
        via_effect, via_process,
        "fmap on a `None` carrier diverges between the two witnesses"
    );
}

/// `fmap` consistency on a command (`RelayTo`) carrier — the case that used to diverge. Both
/// witnesses are now total: the command is preserved (its value leaf mapped through the tree).
#[test]
fn fmap_on_command_agrees_across_witnesses() {
    let cmd = || CausalEffect::relay_to(2, CausalEffect::value(7_i32));
    let val: PropagatingEffect<i32> = PropagatingEffect::new(Ok(cmd()), (), None, EffectLog::new());
    let proc: PropagatingProcess<i32, (), ()> = val.clone();

    let via_effect = <EffectW as Functor<EffectW>>::fmap(val, |x| x + 1);
    let via_process = <ProcessW as Functor<ProcessW>>::fmap(proc, |x| x + 1);

    // The command survives with its target intact and its leaf mapped: RelayTo(2, value(8)).
    assert_eq!(via_effect.command_target(), Some(2));
    assert_eq!(
        via_effect, via_process,
        "fmap on a command carrier diverges between the two witnesses"
    );
}

// NOTE: a `Monad::bind` consistency test across the two witnesses was removed.
// `PropagatingProcessWitness` no longer implements the value-only `Monad` trait, because that bind
// cannot thread the Markovian `State` channel. The canonical state-threading bind is the
// `CausalMonad` trait; see `types/causal_monad/causal_monad_tests.rs`.

/// `Pure::pure` consistency. Lifting a value through either witness produces
/// the same carrier.
#[test]
fn pure_agrees_across_witnesses() {
    let via_effect: PropagatingEffect<i32> = EffectW::pure(99);
    let via_process: PropagatingProcess<i32, (), ()> = ProcessW::pure(99);

    assert_eq!(
        via_effect, via_process,
        "pure diverges between PropagatingEffectWitness and PropagatingProcessWitness"
    );
}
