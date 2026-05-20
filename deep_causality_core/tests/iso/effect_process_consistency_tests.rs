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
    CausalityError, EffectLog, PropagatingEffect, PropagatingEffectWitness, PropagatingProcess,
    PropagatingProcessWitness,
};
use deep_causality_haft::{Functor, Monad, Pure};

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

/// `Monad::bind` consistency. The two `bind` impls are also written
/// independently; this test pins that they sequence operations identically
/// on the shared carrier.
#[test]
fn bind_agrees_across_witnesses() {
    let val: PropagatingEffect<i32> = EffectW::pure(10);
    let proc: PropagatingProcess<i32, (), ()> = val.clone();

    // The continuation must return the SAME concrete carrier type — which it
    // does, because PropagatingEffect<T> and PropagatingProcess<T, (), ()>
    // are type aliases for the same `CausalEffectPropagationProcess`.
    let k = |x: i32| -> PropagatingEffect<i32> { EffectW::pure(x + 100) };

    let via_effect = <EffectW as Monad<EffectW>>::bind(val, k);
    let via_process = <ProcessW as Monad<ProcessW>>::bind(proc, k);

    assert_eq!(
        via_effect, via_process,
        "bind diverges between PropagatingEffectWitness and PropagatingProcessWitness \
         on the shared carrier"
    );
}

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
