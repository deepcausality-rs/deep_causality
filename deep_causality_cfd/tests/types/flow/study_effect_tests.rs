/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `StudyEffect` substrate (the CDL effect pattern): monadic composition obeys the laws,
//! the first error short-circuits and stays tagged with its verb, and non-fatal warnings
//! accumulate through the chain into the terminal.

use deep_causality_cfd::{
    StudyEffect, StudyEffectWitness, StudyError, StudyWarning, StudyWarningLog,
};
use deep_causality_haft::{Applicative, Functor, Monad, Pure};
use deep_causality_physics::PhysicsError;

/// The hidden HKT witness, named here only to drive its lawful instances directly.
type Witness = StudyEffectWitness<StudyError, StudyWarningLog>;

/// Unwrap the effect to its `Ok` value, asserting no error and returning the warning messages.
fn ok_parts<T>(e: StudyEffect<T>) -> (T, Vec<String>) {
    let (inner, warnings) = e.into_parts();
    let value = inner.unwrap_or_else(|err| panic!("expected Ok, got {err}"));
    let msgs = warnings.entries().iter().map(|w| w.message().to_string()).collect();
    (value, msgs)
}

#[test]
fn left_identity_holds() {
    // pure(a).and_then(f) == f(a)
    let f = |x: i32| StudyEffect::pure(x * 2);
    let lhs = StudyEffect::pure(21).and_then(f);
    let rhs = f(21);
    assert_eq!(ok_parts(lhs).0, ok_parts(rhs).0);
}

#[test]
fn right_identity_holds() {
    // m.and_then(pure) == m
    let (lhs, _) = ok_parts(StudyEffect::pure(7).and_then(StudyEffect::pure));
    assert_eq!(lhs, 7);
}

#[test]
fn associativity_holds() {
    // m.and_then(f).and_then(g) == m.and_then(|x| f(x).and_then(g))
    let f = |x: i32| StudyEffect::pure(x + 1);
    let g = |x: i32| StudyEffect::pure(x * 3);
    let lhs = StudyEffect::pure(10).and_then(f).and_then(g);
    let rhs = StudyEffect::pure(10).and_then(|x| f(x).and_then(g));
    assert_eq!(ok_parts(lhs).0, ok_parts(rhs).0);
}

#[test]
fn map_transforms_the_value_and_keeps_warnings() {
    let (value, warnings) = ok_parts(
        StudyEffect::pure(4)
            .warn(StudyWarning::Generic("noted".into()))
            .map(|x| x + 1),
    );
    assert_eq!(value, 5);
    assert_eq!(warnings, vec!["noted".to_string()]);
}

#[test]
fn the_first_error_short_circuits_and_keeps_its_verb_tag() {
    let seed: StudyEffect<i32> = StudyEffect::from_result(Err(StudyError::in_stage(
        "sweep",
        PhysicsError::CalculationError("case 3 diverged".into()),
    )));

    // The continuation must NOT run once the effect is errored.
    let chained = seed.and_then(|_| -> StudyEffect<i32> {
        panic!("and_then ran on an errored effect");
    });

    let (inner, _warnings) = chained.into_parts();
    let err = inner.expect_err("stays errored");
    assert_eq!(err.stage(), "sweep", "the verb tag survives the chain");
    assert!(
        format!("{err}").contains("sweep"),
        "the rendered error names the verb: {err}"
    );
}

#[test]
fn the_witness_carries_lawful_functor_applicative_monad_instances() {
    // Functor::fmap transforms the value, preserving warnings.
    let fmapped = Witness::fmap(StudyEffect::pure(20), |x: i32| x + 1);
    assert_eq!(ok_parts(fmapped).0, 21);

    // Applicative::pure + apply: a wrapped function applied to a wrapped value.
    let applied = Witness::apply(Witness::pure(|x: i32| x * 2), Witness::pure(21));
    assert_eq!(ok_parts(applied).0, 42);

    // Monad::bind sequences and merges warnings; an errored left short-circuits.
    let bound = Witness::bind(StudyEffect::pure(3), |x: i32| StudyEffect::pure(x * 10));
    assert_eq!(ok_parts(bound).0, 30);

    let errored: StudyEffect<i32> = StudyEffect::from_result(Err(StudyError::in_stage(
        "reduce",
        PhysicsError::CalculationError("x".into()),
    )));
    let short: StudyEffect<i32> =
        Witness::bind(errored, |_: i32| panic!("bind ran on an errored effect"));
    assert!(short.into_parts().0.is_err());
}

#[test]
fn warnings_accumulate_in_order_through_the_chain() {
    let effect = StudyEffect::pure(0)
        .warn(StudyWarning::Data("force_load override".into()))
        .and_then(|v| StudyEffect::pure(v).warn(StudyWarning::Case("candidate clamped".into())));

    let (_value, msgs) = ok_parts(effect);
    assert_eq!(
        msgs,
        vec![
            "force_load override".to_string(),
            "candidate clamped".to_string()
        ],
        "warnings from both stages reach the terminal, in order"
    );
}
