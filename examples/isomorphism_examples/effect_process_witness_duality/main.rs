/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # PropagatingEffect / PropagatingProcess Witness-Duality Showcase
//!
//! `PropagatingEffect<T>` and `PropagatingProcess<T, (), ()>` are both type
//! aliases for the same concrete carrier:
//!
//! ```text
//! CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>
//! ```
//!
//! At the value level they are interchangeable.What the codebase
//! actually ships is **two separate HKT witnesses** for the same carrier:
//!
//! - `PropagatingEffectWitness<CausalityError, EffectLog>`
//! - `PropagatingProcessWitness<(), ()>`
//!
//! Each carries its own `Functor` / `Applicative` / `Monad` impl, written
//! independently. The witnesses are different *dispatch paths* into the
//! same concrete operations.
//!
//! ## What this example demonstrates
//!
//! A practitioner can write a generic pipeline that operates on the carrier
//! and choose **either witness path** at the call site. The two paths must
//! produce identical results — that's the consistency property the change
//! pins via a direct `assert_eq!` test rather than through any iso wrapper.
//!
//! This example shows three things:
//!
//! 1. Building one carrier value and dispatching it through both
//!    `fmap` paths. Both produce byte-identical outputs.
//! 2. A generic pipeline parameterised over an arbitrary `Functor` witness
//!    that accepts EITHER witness path interchangeably.
//! 3. The dual-witness ergonomic pattern: when a downstream API takes
//!    `PropagatingProcess<T, (), ()>` and you have a `PropagatingEffect<T>`
//!    in hand, no conversion is needed — they are the same value.
//!
//! ## Why no iso trait here
//!
//! An iso requires a bijection. The Markovian / non-Markovian distinction
//! (statefull vs. stateless) is fundamentally lossy when state is
//! non-trivial. The trivial `S = (), C = ()` case is identity (type alias
//! collapse), which adds nothing the compiler doesn't already know. So
//! this example uses the carrier and the two `Functor` impls directly,
//! with no `NaturalIso` trait between them.
//!

use deep_causality_core::{
    CausalityError, EffectLog, PropagatingEffect, PropagatingEffectWitness, PropagatingProcess,
    PropagatingProcessWitness,
};
use deep_causality_haft::{Functor, HKT, Monad, Pure};

fn main() {
    println!("=== PropagatingEffect / PropagatingProcess Witness Duality ===\n");

    // ---------------------------------------------------------------------
    // Build one carrier value via the `PropagatingEffectWitness::pure` entry.
    // ---------------------------------------------------------------------
    let eff: PropagatingEffect<i32> =
        PropagatingEffectWitness::<CausalityError, EffectLog>::pure(42);
    println!(
        "Initial value (via PropagatingEffectWitness::pure): {:?}",
        eff
    );

    // The same value, viewed through the PropagatingProcessWitness type alias.
    // No conversion: the types are literally equal.
    let proc: PropagatingProcess<i32, (), ()> = eff.clone();
    println!("Same value, ascribed as PropagatingProcess: {:?}", proc);
    println!();

    // ---------------------------------------------------------------------
    // Dispatch the same value through both `fmap` paths.
    // ---------------------------------------------------------------------
    println!("--- Two witness paths, same operation ---");
    let h = |x: i32| x * 2;

    let via_effect =
        <PropagatingEffectWitness<CausalityError, EffectLog> as Functor<_>>::fmap(eff.clone(), h);
    let via_process = <PropagatingProcessWitness<(), ()> as Functor<_>>::fmap(proc.clone(), h);

    println!("  via PropagatingEffectWitness:  {:?}", via_effect);
    println!("  via PropagatingProcessWitness: {:?}", via_process);
    assert_eq!(
        via_effect, via_process,
        "Functor impls on the two witnesses disagree on the shared carrier"
    );
    println!("  -> byte-identical");
    println!();

    // Same exercise with `bind`.
    let k = |x: i32| -> PropagatingEffect<i32> {
        PropagatingEffectWitness::<CausalityError, EffectLog>::pure(x + 100)
    };
    let bound_via_effect =
        <PropagatingEffectWitness<CausalityError, EffectLog> as Monad<_>>::bind(eff.clone(), k);
    let bound_via_process = <PropagatingProcessWitness<(), ()> as Monad<_>>::bind(proc.clone(), k);
    println!(
        "  bind via PropagatingEffectWitness:  {:?}",
        bound_via_effect
    );
    println!(
        "  bind via PropagatingProcessWitness: {:?}",
        bound_via_process
    );
    assert_eq!(
        bound_via_effect, bound_via_process,
        "Monad impls on the two witnesses disagree on the shared carrier"
    );
    println!("  -> byte-identical");
    println!();

    // ---------------------------------------------------------------------
    // Generic pipeline: parameterised over an arbitrary Functor witness.
    // Callers pick which witness path; the function doesn't care.
    // ---------------------------------------------------------------------
    println!("--- Generic pipeline over either witness ---");

    fn doubled_via<W>(input: W::Type<i32>) -> W::Type<i32>
    where
        W: HKT<Constraint = deep_causality_haft::NoConstraint> + Functor<W>,
        i32: deep_causality_haft::Satisfies<W::Constraint>,
    {
        <W as Functor<_>>::fmap(input, |x| x * 2)
    }

    // Same carrier value, two dispatches.
    let result_effect =
        doubled_via::<PropagatingEffectWitness<CausalityError, EffectLog>>(eff.clone());
    let result_process = doubled_via::<PropagatingProcessWitness<(), ()>>(proc.clone());

    println!(
        "  doubled via PropagatingEffectWitness:  {:?}",
        result_effect
    );
    println!(
        "  doubled via PropagatingProcessWitness: {:?}",
        result_process
    );
    assert_eq!(result_effect, result_process);
    println!("  -> identical results from either witness path\n");

    // ---------------------------------------------------------------------
    // The practical payoff: downstream APIs
    // ---------------------------------------------------------------------
    println!("--- Practical use: passing across an API boundary ---");
    let answer = consume_a_process(eff);
    println!(
        "  consume_a_process accepted a PropagatingEffect directly: {:?}",
        answer
    );

    println!("\n--- Summary ---");
    println!("- The carrier is one type with two HKT witnesses.");
    println!("- Two `Functor`/`Monad` impl paths exist, written independently.");
    println!("- Both paths produce identical results on the shared carrier.");
    println!("- Pinned by a direct consistency test in the iso-traits change,");
}
/// A downstream API that wants a `PropagatingProcess<T, (), ()>`. Callers
/// holding a `PropagatingEffect<T>` pass it through directly — they are the
/// same type. No iso conversion, no method call.
fn consume_a_process(p: PropagatingProcess<i32, (), ()>) -> i32 {
    // Trivial example: extract the value, ignoring error and log.
    p.value.into_value().unwrap_or(0)
}
