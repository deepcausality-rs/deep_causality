/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the shared assertion helpers.
//!
//! These exist because the helpers are the instrument every numeric suite is measured with, and an
//! instrument that always reads "pass" is worse than none: it makes 376 tests agree about nothing.
//! Mutation testing showed that directly — replacing each helper's body with `()` left every suite
//! green, because no test asked whether an assertion *fails* when it should.
//!
//! So each helper is checked from both sides: it accepts what it should accept, and it panics on
//! what it should reject. The rejection half is what the mutants could not survive.
//!
//! `catch_unwind` is the mechanism. It needs `std`, which the test tree has, and the panic hook is
//! silenced around each call so a deliberate panic does not print a backtrace and read as a
//! failure.

use deep_causality_num::Float106;
use deep_causality_stats::utils_tests::assertions::{
    assert_close, assert_exact, assert_no_finite_answer, expect_empty_input,
    expect_insufficient_samples,
};
use deep_causality_stats::{StatsError, mean, variance};

/// Runs `f`, returning whether it panicked, without printing the panic.
fn panics(f: impl FnOnce() + std::panic::UnwindSafe) -> bool {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = std::panic::catch_unwind(f);
    std::panic::set_hook(previous);
    outcome.is_err()
}

#[test]
fn assert_close_accepts_a_value_inside_the_tolerance() {
    assert!(!panics(|| assert_close(
        1.000_000_1_f64,
        1.0,
        1e-6,
        "inside"
    )));
    assert!(!panics(|| assert_close(
        1.0_f64,
        1.0,
        0.0,
        "exactly equal needs no slack"
    )));
}

#[test]
fn assert_close_rejects_a_value_outside_the_tolerance() {
    assert!(panics(|| assert_close(1.1_f64, 1.0, 1e-6, "outside")));
    assert!(panics(|| assert_close(-1.0_f64, 1.0, 1e-6, "wrong sign")));
    assert!(panics(|| assert_close(
        f64::NAN,
        1.0,
        1e-6,
        "NaN is not close to anything"
    )));
}

/// The tolerance is relative, with an absolute floor of one.
///
/// Both halves matter. Above one the slack scales with the target, so the same relative error is
/// accepted at any magnitude; at or below one it does not shrink to nothing, which is what keeps
/// the comparison meaningful against a target of zero.
#[test]
fn assert_close_scales_its_tolerance_with_the_target() {
    // 1e6 ± 1 is within 1e-5 relative; 1e6 ± 100 is not.
    assert!(!panics(|| assert_close(
        1_000_001.0_f64,
        1_000_000.0,
        1e-5,
        "scaled"
    )));
    assert!(panics(|| assert_close(
        1_000_100.0_f64,
        1_000_000.0,
        1e-5,
        "beyond the scaled slack"
    )));
    // Against zero the floor of one applies, so 0.5 is inside a tolerance of 1.0 and 2.0 is not.
    assert!(!panics(|| assert_close(
        0.5_f64,
        0.0,
        1.0,
        "absolute floor against zero"
    )));
    assert!(panics(|| assert_close(
        2.0_f64,
        0.0,
        1.0,
        "beyond the floor"
    )));
}

#[test]
fn assert_exact_demands_bit_equality() {
    assert!(!panics(|| assert_exact(2.0_f64, 2.0, "equal")));
    // One ulp apart is close, and not exact. This is the whole difference from `assert_close`.
    // `f64::EPSILON` is the gap at 1.0; the gap at 2.0 is twice it, so adding EPSILON alone
    // rounds straight back to 2.0 and would compare equal.
    let one_ulp_above_two = 2.0_f64 + f64::EPSILON * 2.0;
    assert!(one_ulp_above_two != 2.0, "the fixture must actually differ");
    assert!(panics(move || assert_exact(
        one_ulp_above_two,
        2.0,
        "one ulp is not exact"
    )));
    assert!(panics(|| assert_exact(
        f64::NAN,
        f64::NAN,
        "NaN is not equal to itself"
    )));
}

#[test]
fn assert_exact_holds_at_every_precision() {
    assert!(!panics(|| assert_exact(3.0_f32, 3.0, "f32")));
    assert!(!panics(|| assert_exact(
        Float106::from(3.0),
        Float106::from(3.0),
        "Float106"
    )));
    assert!(panics(|| assert_exact(
        Float106::from(3.0),
        Float106::from(3.5),
        "Float106 differing"
    )));
}

#[test]
fn expect_empty_input_accepts_only_that_variant() {
    let empty: Vec<f64> = Vec::new();
    assert!(!panics(move || expect_empty_input(
        mean(&empty),
        "the empty mean"
    )));
    // A different error is not this one.
    assert!(panics(|| expect_empty_input(
        variance(&[1.0_f64]),
        "a single-element variance"
    )));
    // Nor is success.
    assert!(panics(|| expect_empty_input(
        mean(&[1.0_f64, 2.0]),
        "a mean that worked"
    )));
}

#[test]
fn expect_insufficient_samples_accepts_only_that_variant() {
    assert!(!panics(|| expect_insufficient_samples(
        variance(&[1.0_f64]),
        "one observation"
    )));
    let empty: Vec<f64> = Vec::new();
    assert!(panics(move || expect_insufficient_samples(
        variance(&empty),
        "no observations is a different complaint"
    )));
    assert!(panics(|| expect_insufficient_samples(
        variance(&[1.0_f64, 2.0]),
        "a variance that worked"
    )));
}

/// The non-finite helper accepts two outcomes and refuses one.
///
/// A sample carrying a `NaN` may be refused with `NonFiniteInput` or may return a non-finite
/// value; both say the entry was not silently dropped. Only a finite answer fails, because that is
/// the signature of an implementation that skipped it.
#[test]
fn assert_no_finite_answer_refuses_only_an_invented_finite_value() {
    let nan_sample = [1.0_f64, f64::NAN, 3.0];
    assert!(!panics(move || assert_no_finite_answer(
        mean(&nan_sample),
        "a mean over a NaN"
    )));
    // The documented refusal is equally acceptable, and is the arm no shipped call site reaches:
    // the moments carry no non-finite guard, so they return a non-finite `Ok` rather than refusing.
    // Nothing else exercises this branch, so it is fed directly.
    assert!(!panics(|| assert_no_finite_answer::<f64>(
        Err(StatsError::NonFiniteInput(
            "a non-finite observation has no mean"
        )),
        "the documented refusal"
    )));
    // Any OTHER refusal is not.
    let empty: Vec<f64> = Vec::new();
    assert!(panics(move || assert_no_finite_answer(
        mean(&empty),
        "EmptyInput is not the documented refusal for a non-finite entry"
    )));
    // A finite answer from a clean sample is what the helper exists to catch.
    assert!(panics(|| assert_no_finite_answer(
        mean(&[1.0_f64, 2.0, 3.0]),
        "a finite mean"
    )));
}

#[test]
fn the_helpers_report_the_label_they_were_given() {
    // The label reaches the panic message, so a failure names which assertion broke.
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = std::panic::catch_unwind(|| assert_close(2.0_f64, 1.0, 1e-9, "the label"));
    std::panic::set_hook(previous);
    let payload = outcome.expect_err("the assertion must fail");
    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("");
    assert!(
        message.contains("the label"),
        "the panic message must name the assertion, got {message:?}"
    );
}

#[test]
fn a_result_carrying_an_error_is_still_a_result() {
    // The `expect_*` helpers take a `Result`, so a caller can hand them either arm.
    let ok: Result<f64, StatsError> = Ok(1.0);
    assert!(panics(move || expect_empty_input(ok, "an Ok")));
}
