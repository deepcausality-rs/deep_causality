/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::Endomorphism;
use deep_causality_haft::FnMorphism;

fn inc(x: i32) -> i32 {
    x + 1
}

/// Decrement toward a fixpoint at zero: `max(x - 1, 0)`.
fn dec_to_zero(x: i32) -> i32 {
    if x > 0 { x - 1 } else { 0 }
}

#[test]
fn test_iterate_n_applies_exactly_n_times() {
    let f: fn(i32) -> i32 = inc;
    let r = <FnMorphism as Endomorphism<FnMorphism>>::iterate_n(&f, 0, 5);
    assert_eq!(r, 5);
}

#[test]
fn test_iterate_n_zero_is_identity() {
    let f: fn(i32) -> i32 = inc;
    let r = <FnMorphism as Endomorphism<FnMorphism>>::iterate_n(&f, 7, 0);
    assert_eq!(r, 7);
}

#[test]
fn test_iterate_to_fixpoint_reaches_and_reports_convergence() {
    let f: fn(i32) -> i32 = dec_to_zero;
    let (value, converged) =
        <FnMorphism as Endomorphism<FnMorphism>>::iterate_to_fixpoint(&f, 5, 100);
    assert_eq!(value, 0);
    assert!(converged);
    // The returned value is a true fixpoint: applying once more leaves it unchanged.
    assert_eq!(
        <FnMorphism as deep_causality_haft::Morphism<FnMorphism>>::apply(&f, value),
        value
    );
}

#[test]
fn test_iterate_to_fixpoint_step_bound_reports_non_convergence() {
    let f: fn(i32) -> i32 = inc; // never reaches a fixpoint
    let (value, converged) =
        <FnMorphism as Endomorphism<FnMorphism>>::iterate_to_fixpoint(&f, 0, 3);
    assert!(!converged);
    assert_eq!(value, 3); // stopped after exactly max_steps applications
}

#[test]
fn test_iterate_until_returns_first_value_meeting_predicate() {
    let f: fn(i32) -> i32 = inc;
    let (value, met) =
        <FnMorphism as Endomorphism<FnMorphism>>::iterate_until(&f, 0, |x| *x >= 10, 100);
    assert!(met);
    assert_eq!(value, 10);
}

#[test]
fn test_iterate_until_predicate_true_initially() {
    let f: fn(i32) -> i32 = inc;
    let (value, met) =
        <FnMorphism as Endomorphism<FnMorphism>>::iterate_until(&f, 42, |x| *x >= 10, 100);
    assert!(met);
    assert_eq!(value, 42); // no application needed
}

#[test]
fn test_iterate_until_step_bound_reports_unmet() {
    let f: fn(i32) -> i32 = inc;
    let (value, met) =
        <FnMorphism as Endomorphism<FnMorphism>>::iterate_until(&f, 0, |x| *x >= 100, 5);
    assert!(!met);
    assert_eq!(value, 5);
}
