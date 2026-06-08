/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_calculus::{EndoArrow, Euler};

#[test]
fn test_iterate_to_fixpoint_converges() {
    // step(x) = x + (target − x)·1 = target → a fixpoint after one step.
    let target = 5.0_f64;
    let step = Euler::new(1.0_f64, move |x: &f64| target - *x);
    let (val, converged) = step.iterate_to_fixpoint(0.0, 100);
    assert!(converged);
    assert!((val - target).abs() < 1e-12);
}

#[test]
fn test_iterate_to_fixpoint_bound_hit() {
    // step(x) = x + 1 never fixpoints; the bound is hit.
    let step = Euler::new(1.0_f64, |_: &f64| 1.0);
    let (val, converged) = step.iterate_to_fixpoint(0.0, 5);
    assert!(!converged);
    assert!((val - 5.0).abs() < 1e-12); // 0 + 5·(+1)
}

#[test]
fn test_iterate_until_event() {
    // step(x) = x − 1; stop when x ≤ 0. From 5, reached at step 5.
    let step = Euler::new(1.0_f64, |_: &f64| -1.0);
    let (val, met) = step.iterate_until(5.0, |x| *x <= 0.0, 100);
    assert!(met);
    assert!((val - 0.0).abs() < 1e-12);
}

#[test]
fn test_iterate_until_bound_hit() {
    let step = Euler::new(1.0_f64, |_: &f64| -1.0);
    let (val, met) = step.iterate_until(5.0, |x| *x <= 0.0, 2);
    assert!(!met);
    assert!((val - 3.0).abs() < 1e-12); // 5 − 2
}

#[test]
fn test_iterate_until_predicate_true_initially() {
    let step = Euler::new(1.0_f64, |_: &f64| -1.0);
    let (val, met) = step.iterate_until(-1.0, |x| *x <= 0.0, 10);
    assert!(met);
    assert_eq!(val, -1.0); // predicate holds at the start; no steps taken
}
