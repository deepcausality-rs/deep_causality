/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Bernoulli proportion and its sampling error.
//!
//! The oracles are hand-evaluated closed forms: `p = 1/2` over `n = 100` gives `√(1/400) = 1/20`,
//! and `p = 1/4` over `n = 64` gives `√((1/4)(3/4)/64) = √(3/1024)`.
//!
//! Neither is exact in binary — `1/20` is not a dyadic fraction — so they are asserted at the
//! `literal` tolerance rather than `native`. That row exists for exactly this: a decimal written
//! into the source crosses through `f64`, so at `Float106` the expected value is already wrong by
//! about 1e-18 and a tighter bound would be testing the literal rather than the arithmetic.

use deep_causality_algebra::RealField;
use deep_causality_num::{Float106, FromPrimitive, lift};
use deep_causality_stats::utils_tests::precision::{F32, F64, F106};
use deep_causality_stats::{StatsErrorEnum, bernoulli_proportion, bernoulli_standard_error};

fn err_of<T>(r: Result<T, deep_causality_stats::StatsError>) -> StatsErrorEnum {
    match r {
        Ok(_) => panic!("expected a typed error"),
        Err(e) => e.0,
    }
}

/// `p = 1/2` over 100 trials is `√(0.25/100) = 1/20`.
fn check_half_over_a_hundred<T: RealField + FromPrimitive>(tol: f64) {
    let se = bernoulli_standard_error(lift::<T>(0.5), 100).expect("a proportion in range");
    let want = lift::<T>(0.05);
    assert!(
        (se - want).abs() <= lift::<T>(tol),
        "sqrt(p(1-p)/n) at p = 1/2, n = 100 is exactly 0.05"
    );
}

#[test]
fn test_bernoulli_standard_error_at_a_half() {
    check_half_over_a_hundred::<f32>(F32.literal);
    check_half_over_a_hundred::<f64>(F64.literal);
    check_half_over_a_hundred::<Float106>(F106.literal);
}

/// The width shrinks as `1/√n`: quadrupling the trials halves it, exactly.
#[test]
fn test_bernoulli_standard_error_shrinks_as_one_over_root_n() {
    let at_100 = bernoulli_standard_error::<f64>(0.5, 100).unwrap();
    let at_400 = bernoulli_standard_error::<f64>(0.5, 400).unwrap();
    assert!(
        (at_100 / at_400 - 2.0).abs() < 1e-12,
        "four times the trials halves the width: {at_100} vs {at_400}"
    );
}

/// Both endpoints are accepted and give exactly zero — the observed spread of a sample that never
/// varied. That is the answer to the question asked, not a claim that the truth is certain.
#[test]
fn test_bernoulli_standard_error_is_zero_at_both_endpoints() {
    assert_eq!(bernoulli_standard_error::<f64>(0.0, 50).unwrap(), 0.0);
    assert_eq!(bernoulli_standard_error::<f64>(1.0, 50).unwrap(), 0.0);
}

/// Outside the unit interval `p(1 − p)` is negative and its root is not real, so it is refused
/// rather than returned as a NaN a caller may never inspect.
#[test]
fn test_bernoulli_standard_error_refuses_a_proportion_outside_the_unit_interval() {
    assert!(matches!(
        err_of(bernoulli_standard_error::<f64>(-0.01, 10)),
        StatsErrorEnum::NegativeProbability(_)
    ));
    assert!(matches!(
        err_of(bernoulli_standard_error::<f64>(1.01, 10)),
        StatsErrorEnum::NegativeProbability(_)
    ));
    assert!(matches!(
        err_of(bernoulli_standard_error::<f64>(f64::NAN, 10)),
        StatsErrorEnum::NonFiniteInput(_)
    ));
}

/// No trials means no sampling distribution, so there is no width to report.
#[test]
fn test_bernoulli_standard_error_refuses_zero_trials() {
    assert!(matches!(
        err_of(bernoulli_standard_error::<f64>(0.5, 0)),
        StatsErrorEnum::EmptyInput(_)
    ));
}

/// From counts: 16 of 64 is exactly `1/4`, and the width is `√(3/1024)`.
#[test]
fn test_bernoulli_proportion_from_counts() {
    let (p, se) = bernoulli_proportion::<f64>(16, 64).unwrap();
    assert_eq!(p, 0.25);
    assert!((se - (3.0_f64 / 1024.0).sqrt()).abs() < 1e-15);
}

/// More hits than trials is a caller error the counts can still name; after the division only
/// `p > 1` survives, which cannot say which count was wrong.
#[test]
fn test_bernoulli_proportion_refuses_more_hits_than_trials() {
    assert!(matches!(
        err_of(bernoulli_proportion::<f64>(11, 10)),
        StatsErrorEnum::DimensionMismatch(_)
    ));
    assert!(matches!(
        err_of(bernoulli_proportion::<f64>(0, 0)),
        StatsErrorEnum::EmptyInput(_)
    ));
}
