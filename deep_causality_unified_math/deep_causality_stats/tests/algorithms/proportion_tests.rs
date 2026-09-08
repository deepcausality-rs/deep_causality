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
use deep_causality_num::{BFloat16, Float106, FromPrimitive, lift};
use deep_causality_stats::utils_tests::precision::{BF16, F32, F64, F106};
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

// ---------------------------------------------------------------------------------------------
// Counts wider than the scalar's significand
// ---------------------------------------------------------------------------------------------

/// Provenance: the `BFloat16` grid, and the closed form `SE = √(p(1 − p)/n)` evaluated by hand on
/// the counts.
///
/// `BFloat16` carries eight significand bits, so its spacing in `[0.5, 1)` is `2⁻⁸ = 0.00390625`
/// and the values either side of `1001/1002 = 0.999002` are `0.99609375` and `1`. The nearer of
/// the two is `1`, by `0.000998` against `0.002908`, so the correctly rounded proportion is
/// exactly one — and `1 − p̂` is then exactly zero, which is *not* the sampling width of a sample
/// that saw one miss in a thousand and two.
///
/// The width from the counts: `p̂(1 − p̂)/n = (1001/1002)(1/1002)/1002 = 1001/1002³`, and
/// `1002³ = 1 006 012 008`, so the ratio is `9.95018e-7` and its root is `9.97506e-4`. That
/// literal is the expectation below; the tolerance is the `BFloat16` row, because the answer
/// itself is only representable there to two decimal digits.
///
/// The proportion is asserted separately at `257/259`, where the two roundings do not merely lose
/// a place but move the answer: `257` rounds to `256` and `259` to `260` on the type's grid of
/// even integers above 256, and `256/260 = 0.984375` is four places below the correctly rounded
/// `0.9921875` — which is `254/256`, a value of the type, and the nearer of the two neighbours of
/// `257/259 = 0.992278` by `0.00009` against `0.00380`.
#[test]
fn test_bernoulli_proportion_from_counts_wider_than_the_significand() {
    let (p, se) = bernoulli_proportion::<BFloat16>(1001, 1002).expect("1001 of 1002 is a sample");
    assert_eq!(
        p.to_f64(),
        1.0,
        "1001/1002 rounds to one on the BFloat16 grid"
    );
    // √(1001/1002³) = √9.950179e-7 = 9.975059e-4.
    let want = 9.975_058_6e-4;
    let got = se.to_f64();
    assert!(
        (got - want).abs() <= want * BF16.native,
        "the width of 1001 hits in 1002 trials came back as {got:?}, not {want:?}"
    );

    let (p, _) = bernoulli_proportion::<BFloat16>(257, 259).expect("257 of 259 is a sample");
    assert_eq!(
        p.to_f64(),
        0.992_187_5,
        "257/259 did not round to the nearer of its two BFloat16 neighbours"
    );
}

/// Provenance: an algebraic invariant — `p̂` is a proportion, so it lies in `[0, 1]`, and the width
/// `√(p̂(1 − p̂)/n)` is zero exactly at the two endpoints and strictly positive between them.
///
/// Swept over every count of a hundred trials and every count of 1002, at the scalar whose
/// significand is narrower than either. A width that collapses to zero away from the endpoints is
/// the failure this sweep is for; a proportion outside `[0, 1]` would be the other.
#[test]
fn test_bernoulli_proportion_width_vanishes_only_at_the_endpoints() {
    for trials in [100u64, 1002] {
        for hits in 0..=trials {
            let (p, se) =
                bernoulli_proportion::<BFloat16>(hits, trials).expect("a valid count pair");
            let p = p.to_f64();
            let se = se.to_f64();
            assert!(
                (0.0..=1.0).contains(&p),
                "{hits}/{trials} gave a proportion of {p}, outside [0, 1]"
            );
            if hits == 0 || hits == trials {
                assert_eq!(se, 0.0, "{hits}/{trials} is an endpoint and has no width");
            } else {
                assert!(
                    se > 0.0,
                    "{hits}/{trials} is not an endpoint, yet its width came back as {se}"
                );
            }
        }
    }
}

/// Provenance: the same closed form at a scalar wide enough to hold the counts exactly, so the
/// route through the counts must not disturb the answer it already gave.
///
/// `1001/1002³` again, to more places: `9.95017944159569e-7`, whose root is
/// `9.97505861716897e-4`. `f64` holds every count under `2⁵³` exactly, so the only rounding here
/// is the arithmetic's.
#[test]
fn test_bernoulli_proportion_at_a_wide_scalar_is_unchanged() {
    let (p, se) = bernoulli_proportion::<f64>(1001, 1002).expect("1001 of 1002 is a sample");
    assert!(
        (p - 1001.0 / 1002.0).abs() <= F64.native,
        "the proportion moved: {p}"
    );
    let want = 9.975_058_617_168_97e-4;
    assert!(
        (se - want).abs() <= want * F64.native,
        "the width moved: {se}, not {want}"
    );
}
