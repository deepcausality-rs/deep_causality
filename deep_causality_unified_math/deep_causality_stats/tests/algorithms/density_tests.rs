/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Phase-2 suite for [`gaussian_log_density`].
//!
//! The contract under test is the one written in `src/algorithms/density.rs`:
//!
//! ```text
//! log N(x | mu, sigma^2) = -1/2 * log(2 * pi * sigma^2) - (x - mu)^2 / (2 * sigma^2)
//! ```
//!
//! with the **third argument the variance**, and a non-positive variance refused with
//! `NonPositiveScale`.
//!
//! # Where the expected numbers come from
//!
//! Every decimal literal in an assertion is one of four things, and each is named at its use site:
//!
//! 1. A hand evaluation of the closed form above at a point where the arithmetic is short,
//!    built from three published constants and nothing else:
//!    * `ln 2  = 0.6931471805599453094172321214582`
//!    * `ln pi = 1.1447298858494001741434273513531`
//!    * hence `ln(2*pi) = ln 2 + ln pi = 1.8378770664093454835606594728113`
//!
//!    Source: Abramowitz & Stegun, *Handbook of Mathematical Functions*, Table 1.1; the same
//!    values are the standard `LN_2` and the logarithm of `PI` in any reference table. Every
//!    other logarithm used below is an integer multiple of `ln 2`, so each expectation is one
//!    addition and one halving away from those two constants and can be re-checked by hand.
//! 2. An algebraic invariant of the density (symmetry, the mode, translation invariance,
//!    a normalised total mass of exactly `1`).
//! 3. A demonstrably different algorithm: the trapezoid rule, which recovers the total mass by
//!    numerical quadrature rather than by re-evaluating the density's own formula.
//! 4. A bound or an ordering, where no closed form is asserted at all.
//!
//! No expectation is produced by calling `gaussian_log_density`, and no expectation retypes the
//! function's formula: `T::ln` and `T::pi` are never called anywhere in this file. `T::exp` is
//! called only in the quadrature test, where exponentiating is the inverse of the operation under
//! test and is what makes the quadrature independent of it, and in the underflow test, where the
//! claim is precisely that `exp` of the result is zero.
//!
//! # Precision
//!
//! Every numeric test is a generic helper over `T: RealField + FromPrimitive`, called once at
//! `f32`, once at `f64` and once at `Float106`, each with its own tolerance. The tolerances are
//! mixed absolute/relative (`|a - e| <= tol * max(1, |e|)`), so the same constant serves the small
//! log-densities near the mode and the large ones far out in the tail.
//!
//! The `Float106` tolerance is deliberately no tighter than `1e-15`. Expectations reach the
//! working scalar through `lift`, which crosses from an `f64` literal, so the expectation itself
//! already carries up to half an `f64` ulp of error. The `Float106` run therefore checks that the
//! double-double path agrees with the same closed form, not that it is more accurate than `f64`;
//! pinning more than that would need a fixture that does not go through `f64`.
//!
//! # Corner-case rows that do not apply
//!
//! * **A (empty input)** and **B (single element)**: the function takes three scalars, not a
//!   collection. There is no length to be zero or one, and `EmptyInput` is unreachable here.
//! * **D (a degenerate index expression)**: the function indexes nothing.
//! * **H (a probability exactly 0 or 1)**: there is no probability argument. A log-density is not
//!   a probability and is unbounded above as the variance shrinks, which
//!   `test_small_positive_variance_is_accepted_and_raises_the_peak` pins directly. The density's
//!   own domain boundary is `variance = 0`, covered under E and F.
//!
//! Row C (two distinct quantities coincide) is the reason this file exists in the shape it does:
//! at `variance = 1` the variance and the standard deviation are the same number, so a suite that
//! only tested there could not tell the two parameterisations apart. Rows E, F, G, I, J and K are
//! covered by the tests named in each section below.

use deep_causality_algebra::{Real, RealField};
use deep_causality_num::lift;
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_stats::utils_tests::precision::{F32, F64, F106};
use deep_causality_stats::{StatsError, StatsErrorEnum, gaussian_log_density};

// ---------------------------------------------------------------------------------------------
// Per-precision tolerances (row K).
// ---------------------------------------------------------------------------------------------

/// Quadrature tolerances. The trapezoid sum below runs over 4001 nodes, so the dominant term is
/// accumulated rounding in the summation (roughly `n * eps`), not the rule's truncation error.
/// For `f32` that is about `4001 * 6e-8 ~= 2.4e-4`, so `1e-3` is the honest bound.
const QUAD_TOL_F32: f64 = 1e-3;
/// For `f64` and `Float106` the accumulated rounding is ~`1e-13` and the truncation of the
/// infinite domain at +/- 10 standard deviations contributes ~`1.5e-23`, so `1e-10` is loose but
/// still a real check.
const QUAD_TOL_F64: f64 = 1e-10;
/// Same reasoning as [`QUAD_TOL_F64`].
const QUAD_TOL_F106: f64 = 1e-10;

// ---------------------------------------------------------------------------------------------
// Assertion helpers.
// ---------------------------------------------------------------------------------------------

/// `|actual - expected| <= tol * max(1, |expected|)`.
///
/// Values are reported through `ToPrimitive::to_f64` because the generic bound does not promise
/// `Debug`.
fn assert_close<T>(actual: T, expected: T, tol: T, what: &str)
where
    T: RealField + FromPrimitive,
{
    let one = lift::<T>(1.0);
    let magnitude = expected.abs();
    let scale = if magnitude > one { magnitude } else { one };
    let diff = (actual - expected).abs();
    assert!(
        diff <= tol * scale,
        "{what}: actual {:?}, expected {:?}, allowed {:?}",
        actual.to_f64(),
        expected.to_f64(),
        (tol * scale).to_f64(),
    );
}

/// The call was refused with [`StatsErrorEnum::NonPositiveScale`].
fn assert_non_positive_scale<T>(result: Result<T, StatsError>, what: &str)
where
    T: RealField + FromPrimitive,
{
    match result {
        Ok(v) => panic!(
            "{what}: expected NonPositiveScale, got Ok({:?})",
            v.to_f64()
        ),
        Err(StatsError(StatsErrorEnum::NonPositiveScale(_))) => {}
        Err(StatsError(other)) => panic!("{what}: expected NonPositiveScale, got {other:?}"),
    }
}

/// The call was refused with [`StatsErrorEnum::NonFiniteInput`].
fn assert_non_finite_input<T>(result: Result<T, StatsError>, what: &str)
where
    T: RealField + FromPrimitive,
{
    match result {
        Ok(v) => panic!("{what}: expected NonFiniteInput, got Ok({:?})", v.to_f64()),
        Err(StatsError(StatsErrorEnum::NonFiniteInput(_))) => {}
        Err(StatsError(other)) => panic!("{what}: expected NonFiniteInput, got {other:?}"),
    }
}

/// The call was refused, with either of the two variants that a value which is both non-finite
/// *and* non-positive can legitimately be reported under. Used only for `variance = -inf`.
fn assert_refused_as_non_finite_or_non_positive<T>(result: Result<T, StatsError>, what: &str)
where
    T: RealField + FromPrimitive,
{
    match result {
        Ok(v) => panic!("{what}: expected an error, got Ok({:?})", v.to_f64()),
        Err(StatsError(StatsErrorEnum::NonFiniteInput(_))) => {}
        Err(StatsError(StatsErrorEnum::NonPositiveScale(_))) => {}
        Err(StatsError(other)) => {
            panic!("{what}: expected NonFiniteInput or NonPositiveScale, got {other:?}")
        }
    }
}

// ---------------------------------------------------------------------------------------------
// 1. The closed form at the mode of the standard normal.
// ---------------------------------------------------------------------------------------------

fn at_the_mean_with_unit_variance<T>(tol: f64)
where
    T: RealField + FromPrimitive,
{
    // Closed form at x = mu, sigma^2 = 1:
    //
    //   log N(mu | mu, 1) = -1/2 * log(2 * pi * 1) - 0 / 2 = -1/2 * ln(2*pi)
    //
    // Hand evaluation, from the two published constants in the module header:
    //   ln(2*pi) = ln 2 + ln pi
    //            = 0.6931471805599453094172321214582
    //            + 1.1447298858494001741434273513531
    //            = 1.8378770664093454835606594728113
    //   half of that = 0.9189385332046727417803297364057
    // so the expected value is its negation.
    let expected = lift::<T>(-0.918_938_533_204_672_8);

    // The mean is deliberately not zero, so that the second term vanishing is the consequence of
    // x = mu rather than of both being zero.
    let x = lift::<T>(3.5);
    let mean = lift::<T>(3.5);
    let variance = lift::<T>(1.0);

    let got = gaussian_log_density(x, mean, variance).expect("unit variance is positive");
    assert_close(got, expected, lift::<T>(tol), "log N(mu | mu, 1)");

    // The same value again with a zero mean, pinning that only the deviation matters (row F: the
    // location and the deviation are both exactly zero here).
    let got_at_origin = gaussian_log_density(lift::<T>(0.0), lift::<T>(0.0), lift::<T>(1.0))
        .expect("unit variance is positive");
    assert_close(
        got_at_origin,
        expected,
        lift::<T>(tol),
        "log N(0 | 0, 1) equals log N(mu | mu, 1)",
    );
}

#[test]
fn test_at_the_mean_with_unit_variance_is_minus_half_log_two_pi() {
    at_the_mean_with_unit_variance::<f32>(F32.literal);
    at_the_mean_with_unit_variance::<f64>(F64.literal);
    at_the_mean_with_unit_variance::<Float106>(F106.literal);
}

// ---------------------------------------------------------------------------------------------
// 2 and 3. The parameterisation trap (row C: variance and standard deviation coincide at 1, and
//          only there). Both tests assert the variance reading and additionally assert that the
//          answer is NOT the one the standard-deviation reading would give.
// ---------------------------------------------------------------------------------------------

fn variance_not_standard_deviation_at_the_mean<T>(tol: f64)
where
    T: RealField + FromPrimitive,
{
    // At x = mu the second term is zero, so the whole answer is the normalising constant and the
    // two readings of the third argument are separated by it alone.
    //
    // VARIANCE reading, sigma^2 = 4 (so sigma = 2):
    //   -1/2 * ln(2*pi*4) = -1/2 * (ln(2*pi) + 2*ln 2)
    //                     = -1/2 * (1.8378770664093454835606594728113
    //                             + 1.3862943611198906188344642429164)
    //                     = -1/2 *  3.2241714275292361023951237157277
    //                     = -1.6120857137646180511975618578639
    let expected_variance_reading = lift::<T>(-1.612_085_713_764_618);
    //
    // STANDARD-DEVIATION reading, sigma = 4 (so sigma^2 = 16), which this crate does NOT use:
    //   -1/2 * ln(2*pi*16) = -1/2 * (ln(2*pi) + 4*ln 2)
    //                      = -1/2 * (1.8378770664093454835606594728113
    //                              + 2.7725887222397812376689284858328)
    //                      = -1/2 *  4.6104657886491267212295879586441
    //                      = -2.3052328943245633606147939793221
    let rejected_sigma_reading = lift::<T>(-2.305_232_894_324_563_5);

    let got = gaussian_log_density(lift::<T>(1.0), lift::<T>(1.0), lift::<T>(4.0))
        .expect("variance 4 is positive");
    assert_close(
        got,
        expected_variance_reading,
        lift::<T>(tol),
        "log N(mu | mu, sigma^2 = 4)",
    );

    // The two readings are 0.69 apart, far outside any tolerance here, so this is a real
    // separation and not a restatement of the assertion above.
    let separation = (got - rejected_sigma_reading).abs();
    assert!(
        separation > lift::<T>(0.5),
        "third argument was read as the standard deviation, not the variance: got {:?}",
        got.to_f64()
    );

    // The same trap on the other side of 1, where the two readings differ even in sign.
    //
    // VARIANCE reading, sigma^2 = 1/4:
    //   -1/2 * ln(2*pi/4) = -1/2 * (ln(2*pi) - 2*ln 2)
    //                     = -1/2 * (1.8378770664093454835606594728113
    //                             - 1.3862943611198906188344642429164)
    //                     = -1/2 *  0.4515827052894548647261952298949
    //                     = -0.2257913526447274323630976149475
    let expected_quarter = lift::<T>(-0.225_791_352_644_727_44);
    //
    // STANDARD-DEVIATION reading, sigma = 1/4 (so sigma^2 = 1/16):
    //   -1/2 * ln(2*pi/16) = -1/2 * (ln(2*pi) - 4*ln 2)
    //                      = -1/2 * (1.8378770664093454835606594728113
    //                              - 2.7725887222397812376689284858328)
    //                      = -1/2 * -0.9347116558304357541082690130215
    //                      = +0.4673558279152178770541345065108
    //
    // The variance reading is negative and the standard-deviation reading is positive, so the
    // sign alone decides this one.
    let got_quarter = gaussian_log_density(lift::<T>(-2.0), lift::<T>(-2.0), lift::<T>(0.25))
        .expect("variance 0.25 is positive");
    assert_close(
        got_quarter,
        expected_quarter,
        lift::<T>(tol),
        "log N(mu | mu, sigma^2 = 0.25)",
    );
    assert!(
        got_quarter < lift::<T>(0.0),
        "at variance 0.25 the log-density must be negative; a positive value is the \
         standard-deviation reading: got {:?}",
        got_quarter.to_f64()
    );
}

#[test]
fn test_third_argument_is_the_variance_not_the_standard_deviation_at_the_mean() {
    variance_not_standard_deviation_at_the_mean::<f32>(F32.literal);
    variance_not_standard_deviation_at_the_mean::<f64>(F64.literal);
    variance_not_standard_deviation_at_the_mean::<Float106>(F106.literal);
}

fn variance_not_standard_deviation_off_the_mean<T>(tol: f64)
where
    T: RealField + FromPrimitive,
{
    // x = mu + 2, sigma^2 = 4. Now BOTH terms separate the two readings.
    //
    // VARIANCE reading:
    //   -1/2 * ln(2*pi*4) - (2^2) / (2*4)
    //     = -1.6120857137646180511975618578639 - 4/8
    //     = -1.6120857137646180511975618578639 - 0.5
    //     = -2.1120857137646180511975618578639
    // (the first term is the hand evaluation carried over from the test above).
    let expected_variance_reading = lift::<T>(-2.112_085_713_764_618);
    //
    // STANDARD-DEVIATION reading (sigma = 4, sigma^2 = 16):
    //   -1/2 * ln(2*pi*16) - (2^2) / (2*16)
    //     = -2.3052328943245633606147939793221 - 4/32
    //     = -2.3052328943245633606147939793221 - 0.125
    //     = -2.4302328943245633606147939793221
    let rejected_sigma_reading = lift::<T>(-2.430_232_894_324_563_5);

    let got = gaussian_log_density(lift::<T>(7.0), lift::<T>(5.0), lift::<T>(4.0))
        .expect("variance 4 is positive");
    assert_close(
        got,
        expected_variance_reading,
        lift::<T>(tol),
        "log N(mu + 2 | mu, sigma^2 = 4)",
    );

    // 0.318 apart, far outside any tolerance in this file.
    assert!(
        (got - rejected_sigma_reading).abs() > lift::<T>(0.25),
        "third argument was read as the standard deviation, not the variance: got {:?}",
        got.to_f64()
    );
}

#[test]
fn test_third_argument_is_the_variance_not_the_standard_deviation_off_the_mean() {
    variance_not_standard_deviation_off_the_mean::<f32>(F32.literal);
    variance_not_standard_deviation_off_the_mean::<f64>(F64.literal);
    variance_not_standard_deviation_off_the_mean::<Float106>(F106.literal);
}

// ---------------------------------------------------------------------------------------------
// 4. The density integrates to one. Independent algorithm: trapezoid quadrature.
// ---------------------------------------------------------------------------------------------

fn integrates_to_one<T>(tol: f64)
where
    T: RealField + FromPrimitive,
{
    // Composite trapezoid rule for the standard normal over [-10, 10] with h = 0.005, i.e. 4001
    // nodes:
    //
    //   I ~= h * ( f(t_0)/2 + f(t_1) + ... + f(t_{n-1}) + f(t_n)/2 )
    //
    // The expected value is 1: a probability density integrates to one over its support. That is
    // an invariant of the object, not a restatement of the formula, and the quadrature reaches it
    // by a route the function under test does not take.
    //
    // Truncating the real line at +/- 10 standard deviations discards about 1.5e-23 of the mass,
    // which is below every tolerance used here.
    let mean = lift::<T>(0.0);
    let variance = lift::<T>(1.0);
    let half = lift::<T>(0.5);
    let step = 0.005_f64;
    let nodes = 4000usize;

    let mut acc = lift::<T>(0.0);
    for i in 0..=nodes {
        let t = -10.0 + step * (i as f64);
        let log_density =
            gaussian_log_density(lift::<T>(t), mean, variance).expect("unit variance is positive");
        // Exponentiating is the inverse of the operation under test; it is what turns the
        // log-density back into the density the quadrature needs.
        let density = Real::exp(log_density);
        if i == 0 || i == nodes {
            acc += density * half;
        } else {
            acc += density;
        }
    }
    let integral = acc * lift::<T>(step);

    // A normalised density has total mass exactly one.
    assert_close(
        integral,
        lift::<T>(1.0),
        lift::<T>(tol),
        "trapezoid quadrature of exp(log-density) over [-10, 10]",
    );
}

#[test]
fn test_density_integrates_to_one_by_trapezoid_quadrature() {
    integrates_to_one::<f32>(QUAD_TOL_F32);
    integrates_to_one::<f64>(QUAD_TOL_F64);
    integrates_to_one::<Float106>(QUAD_TOL_F106);
}

// ---------------------------------------------------------------------------------------------
// 5. Symmetry about the mean. Invariant, no closed form.
// ---------------------------------------------------------------------------------------------

fn symmetric_about_the_mean<T>(tol: f64)
where
    T: RealField + FromPrimitive,
{
    // The density depends on x only through (x - mu)^2, so log N(mu + d) = log N(mu - d) for
    // every d. Asserted over a family of offsets and at two variances, including a negative mean
    // (row G on the location parameter, which the contract does not constrain).
    for (mean, variance) in [(-3.0_f64, 4.0_f64), (0.0, 1.0), (2.5, 0.25)] {
        for d in [0.0_f64, 0.125, 1.0, 3.0, 12.0] {
            let left =
                gaussian_log_density(lift::<T>(mean - d), lift::<T>(mean), lift::<T>(variance))
                    .expect("variance is positive");
            let right =
                gaussian_log_density(lift::<T>(mean + d), lift::<T>(mean), lift::<T>(variance))
                    .expect("variance is positive");
            assert_close(
                left,
                right,
                lift::<T>(tol),
                "log-density is symmetric about the mean",
            );
        }
    }
}

#[test]
fn test_symmetric_about_the_mean() {
    symmetric_about_the_mean::<f32>(F32.native);
    symmetric_about_the_mean::<f64>(F64.native);
    symmetric_about_the_mean::<Float106>(F106.native);
}

// ---------------------------------------------------------------------------------------------
// 6. The maximum is at x = mean. Ordering, no closed form.
// ---------------------------------------------------------------------------------------------

fn maximum_is_at_the_mean<T>()
where
    T: RealField + FromPrimitive,
{
    // -(x - mu)^2 / (2 sigma^2) is strictly negative for x != mu and zero at x = mu, so the mode
    // is at the mean and the log-density falls off strictly monotonically on either side.
    for (mean, variance) in [(-3.0_f64, 4.0_f64), (0.0, 1.0), (2.5, 0.25)] {
        let peak = gaussian_log_density(lift::<T>(mean), lift::<T>(mean), lift::<T>(variance))
            .expect("variance is positive");

        let mut previous = peak;
        for d in [0.125_f64, 1.0, 3.0, 12.0] {
            for signed in [d, -d] {
                let off = gaussian_log_density(
                    lift::<T>(mean + signed),
                    lift::<T>(mean),
                    lift::<T>(variance),
                )
                .expect("variance is positive");
                assert!(
                    off < peak,
                    "the mode must be at the mean: off-mean {:?} was not below peak {:?}",
                    off.to_f64(),
                    peak.to_f64()
                );
            }
            // Strictly decreasing in |x - mu|.
            let further =
                gaussian_log_density(lift::<T>(mean + d), lift::<T>(mean), lift::<T>(variance))
                    .expect("variance is positive");
            assert!(
                further < previous,
                "the log-density must fall strictly as |x - mu| grows: {:?} was not below {:?}",
                further.to_f64(),
                previous.to_f64()
            );
            previous = further;
        }
    }
}

#[test]
fn test_maximum_is_at_the_mean() {
    maximum_is_at_the_mean::<f32>();
    maximum_is_at_the_mean::<f64>();
    maximum_is_at_the_mean::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// 7 and 8. The documented threshold: variance > 0 (rows E, F, G).
// ---------------------------------------------------------------------------------------------

fn zero_variance_is_refused<T>()
where
    T: RealField + FromPrimitive,
{
    // Row F, and the lower side of the only documented threshold: "A non-positive variance is
    // refused". Zero is non-positive.
    assert_non_positive_scale(
        gaussian_log_density(lift::<T>(0.0), lift::<T>(0.0), lift::<T>(0.0)),
        "variance = 0 at the mean",
    );
    // Also away from the mean, where a naive implementation divides a non-zero numerator by zero
    // and returns -inf rather than an error.
    assert_non_positive_scale(
        gaussian_log_density(lift::<T>(4.0), lift::<T>(1.0), lift::<T>(0.0)),
        "variance = 0 away from the mean",
    );
    // Negative zero is the same point on the real line and must be refused identically.
    assert_non_positive_scale(
        gaussian_log_density(lift::<T>(4.0), lift::<T>(1.0), lift::<T>(-0.0)),
        "variance = -0.0",
    );
}

#[test]
fn test_zero_variance_is_refused_with_non_positive_scale() {
    zero_variance_is_refused::<f32>();
    zero_variance_is_refused::<f64>();
    zero_variance_is_refused::<Float106>();
}

fn negative_variance_is_refused<T>()
where
    T: RealField + FromPrimitive,
{
    // Row G, and the same threshold from below. A negative variance would make the argument of
    // the logarithm negative, so there is no value to return.
    for v in [-1.0_f64, -4.0, -1e-6, -1e12] {
        assert_non_positive_scale(
            gaussian_log_density(lift::<T>(0.5), lift::<T>(0.0), lift::<T>(v)),
            "negative variance",
        );
    }
}

#[test]
fn test_negative_variance_is_refused_with_non_positive_scale() {
    negative_variance_is_refused::<f32>();
    negative_variance_is_refused::<f64>();
    negative_variance_is_refused::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// 9. The accepting side of the threshold (row E), and the log-density's unbounded growth.
// ---------------------------------------------------------------------------------------------

/// `tiny` is a variance close to the bottom of each type's normal range: `1e-30` for `f32`, whose
/// smallest normal is about `1.18e-38`, and `1e-300` for the two `f64`-based types, whose smallest
/// normal is about `2.23e-308`. That is corner-case row J from the underflow side.
fn small_positive_variance_is_accepted<T>(tiny: f64)
where
    T: RealField + FromPrimitive,
{
    let zero = lift::<T>(0.0);
    let unit = gaussian_log_density(zero, zero, lift::<T>(1.0)).expect("variance 1 is positive");

    // Any strictly positive variance is on the accepting side of the threshold, however small.
    let peaked = gaussian_log_density(zero, zero, lift::<T>(tiny))
        .expect("a strictly positive variance is accepted, however small");

    assert!(
        peaked.is_finite(),
        "a tiny but representable variance must give a finite log-density: got {:?}",
        peaked.to_f64()
    );
    // Narrowing a density raises its peak: the mode value 1/sqrt(2*pi*sigma^2) grows without
    // bound as sigma^2 falls. This is an ordering, not a formula.
    assert!(
        peaked > unit,
        "shrinking the variance must raise the peak: {:?} was not above {:?}",
        peaked.to_f64(),
        unit.to_f64()
    );
    // The peak DENSITY exceeds 1 (so its logarithm exceeds 0) exactly when
    // 2*pi*sigma^2 < 1, i.e. sigma^2 < 1/(2*pi) = 0.15915494309189533577.
    // Both `tiny` values are enormously below that, so the log-density at the mode is positive.
    // The bound 0.15915494309189533577 is 1/(2*pi) with 2*pi = 6.28318530717958647693
    // (Abramowitz & Stegun, Table 1.1).
    assert!(
        peaked > lift::<T>(0.0),
        "for variance far below 1/(2*pi) the log-density at the mode must be positive: got {:?}",
        peaked.to_f64()
    );
    // And the unit-variance value is on the other side of that same bound, since 1 > 1/(2*pi).
    assert!(
        unit < lift::<T>(0.0),
        "for variance 1, which is above 1/(2*pi), the log-density at the mode must be negative: \
         got {:?}",
        unit.to_f64()
    );

    // Just below the 1/(2*pi) crossing the log-density at the mode is positive, and just above it
    // is negative: both sides of a threshold that follows from the closed form rather than from
    // the contract's own wording.
    let below =
        gaussian_log_density(zero, zero, lift::<T>(0.15)).expect("variance 0.15 is positive");
    let above =
        gaussian_log_density(zero, zero, lift::<T>(0.17)).expect("variance 0.17 is positive");
    assert!(
        below > lift::<T>(0.0),
        "variance 0.15 < 1/(2*pi): log-density at the mode must be positive, got {:?}",
        below.to_f64()
    );
    assert!(
        above < lift::<T>(0.0),
        "variance 0.17 > 1/(2*pi): log-density at the mode must be negative, got {:?}",
        above.to_f64()
    );
}

#[test]
fn test_small_positive_variance_is_accepted_and_raises_the_peak() {
    small_positive_variance_is_accepted::<f32>(1e-30);
    small_positive_variance_is_accepted::<f64>(1e-300);
    small_positive_variance_is_accepted::<Float106>(1e-300);
}

// ---------------------------------------------------------------------------------------------
// 10, 11 and 12. Non-finite inputs (row I).
//
// The doc comment on `gaussian_log_density` names only the non-positive-variance refusal. The
// reading asserted here is the crate's own error taxonomy: `NonFiniteInput` is documented as "an
// input carried a non-finite value where the statistic has no meaning for one", and a log-density
// at a NaN or infinite argument has no meaning. A NaN variance in particular is NOT non-positive,
// because every comparison against NaN is false, so `NonPositiveScale` cannot be its variant.
// ---------------------------------------------------------------------------------------------

fn non_finite_x_is_refused<T>()
where
    T: RealField + FromPrimitive,
{
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_non_finite_input(
            gaussian_log_density(lift::<T>(bad), lift::<T>(0.0), lift::<T>(1.0)),
            "non-finite x",
        );
    }
}

#[test]
fn test_non_finite_x_is_refused_with_non_finite_input() {
    non_finite_x_is_refused::<f32>();
    non_finite_x_is_refused::<f64>();
    non_finite_x_is_refused::<Float106>();
}

fn non_finite_mean_is_refused<T>()
where
    T: RealField + FromPrimitive,
{
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_non_finite_input(
            gaussian_log_density(lift::<T>(0.0), lift::<T>(bad), lift::<T>(1.0)),
            "non-finite mean",
        );
    }
    // Both non-finite at once, where the deviation would be NaN even though neither difference is
    // representable.
    assert_non_finite_input(
        gaussian_log_density(
            lift::<T>(f64::INFINITY),
            lift::<T>(f64::INFINITY),
            lift::<T>(1.0),
        ),
        "x and mean both +inf",
    );
}

#[test]
fn test_non_finite_mean_is_refused_with_non_finite_input() {
    non_finite_mean_is_refused::<f32>();
    non_finite_mean_is_refused::<f64>();
    non_finite_mean_is_refused::<Float106>();
}

fn non_finite_variance_is_refused<T>()
where
    T: RealField + FromPrimitive,
{
    // NaN is unordered, so it is not caught by the non-positive test; +inf is strictly positive,
    // so it passes the non-positive test outright. Both must still be refused.
    assert_non_finite_input(
        gaussian_log_density(lift::<T>(0.0), lift::<T>(0.0), lift::<T>(f64::NAN)),
        "NaN variance",
    );
    assert_non_finite_input(
        gaussian_log_density(lift::<T>(0.0), lift::<T>(0.0), lift::<T>(f64::INFINITY)),
        "+inf variance",
    );
    // -inf is simultaneously non-finite and non-positive. The doc pins the refusal but not which
    // of the two statements about the input gets reported, so either variant is accepted here and
    // the assertion is that it is refused at all.
    assert_refused_as_non_finite_or_non_positive(
        gaussian_log_density(lift::<T>(0.0), lift::<T>(0.0), lift::<T>(f64::NEG_INFINITY)),
        "-inf variance",
    );
}

#[test]
fn test_non_finite_variance_is_refused() {
    non_finite_variance_is_refused::<f32>();
    non_finite_variance_is_refused::<f64>();
    non_finite_variance_is_refused::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// 13. Why a log-density exists at all: the density underflows, the logarithm does not.
// ---------------------------------------------------------------------------------------------

fn large_deviation_keeps_the_log_finite<T>(tol: f64)
where
    T: RealField + FromPrimitive,
{
    // x - mu = 100, sigma^2 = 1. Closed form:
    //
    //   -1/2 * ln(2*pi) - 100^2 / (2*1)
    //     = -0.9189385332046727417803297364057 - 10000/2
    //     = -0.9189385332046727417803297364057 - 5000
    //     = -5000.9189385332046727417803297364
    //
    // (the constant is the hand evaluation from test 1).
    let expected = lift::<T>(-5_000.918_938_533_205);

    let got = gaussian_log_density(lift::<T>(150.0), lift::<T>(50.0), lift::<T>(1.0))
        .expect("unit variance is positive");

    assert!(
        got.is_finite(),
        "100 standard deviations out, the log-density must still be finite: got {:?}",
        got.to_f64()
    );
    assert_close(got, expected, lift::<T>(tol), "log N(mu + 100 | mu, 1)");

    // The point of the whole function: exp(-5000.9) is below the smallest representable positive
    // number of every type here (f32 loses normals below ~1.18e-38 and subnormals below ~1.4e-45;
    // f64 below ~4.9e-324), so the density itself is exactly zero while its logarithm is an
    // ordinary finite number. A density-valued API cannot express this point at all.
    assert!(
        got.exp() == lift::<T>(0.0),
        "exp of the log-density must underflow to zero here: got {:?}",
        got.exp().to_f64()
    );
}

#[test]
fn test_large_deviation_keeps_the_log_finite_while_the_density_underflows() {
    large_deviation_keeps_the_log_finite::<f32>(F32.literal);
    large_deviation_keeps_the_log_finite::<f64>(F64.literal);
    large_deviation_keeps_the_log_finite::<Float106>(F106.literal);
}

// ---------------------------------------------------------------------------------------------
// 14. Row J: the largest separation whose square the type can still hold.
// ---------------------------------------------------------------------------------------------

/// `separation` is chosen per precision so that its square is finite in that type and close to the
/// top of its range: `1e19` for `f32` (square `1e38`, against a maximum of about `3.40e38`), and
/// `1e150` for the two `f64`-based types (square `1e300`, against about `1.80e308`).
///
/// A separation whose square overflows the type is deliberately not asserted: the doc comment says
/// nothing about what the function returns when the squared deviation leaves the type's range, so
/// there is no documented answer to pin. This test stays inside the representable range and
/// asserts only what follows from the closed form there.
fn extreme_separation<T>(separation: f64)
where
    T: RealField + FromPrimitive,
{
    let zero = lift::<T>(0.0);
    let one = lift::<T>(1.0);

    let far =
        gaussian_log_density(lift::<T>(separation), zero, one).expect("unit variance is positive");
    let near =
        gaussian_log_density(lift::<T>(100.0), zero, one).expect("unit variance is positive");

    assert!(
        far.is_finite(),
        "a separation whose square is representable must give a finite log-density, not -inf: \
         got {:?}",
        far.to_f64()
    );
    assert!(
        far < near,
        "the log-density must keep falling out to the type's own range: {:?} was not below {:?}",
        far.to_f64(),
        near.to_f64()
    );
    assert!(
        far < zero,
        "the log-density far out in the tail must be negative: got {:?}",
        far.to_f64()
    );
    // Symmetry survives all the way out (invariant, restated at the extreme).
    let mirrored =
        gaussian_log_density(lift::<T>(-separation), zero, one).expect("unit variance is positive");
    assert!(
        (far - mirrored).abs() <= (far.abs()) * lift::<T>(1e-5),
        "symmetry must hold at the extreme separation: {:?} vs {:?}",
        far.to_f64(),
        mirrored.to_f64()
    );
}

#[test]
fn test_extreme_separation_at_each_types_own_range() {
    extreme_separation::<f32>(1e19);
    extreme_separation::<f64>(1e150);
    extreme_separation::<Float106>(1e150);
}

// ---------------------------------------------------------------------------------------------
// 15. Translation invariance, and negative locations (row G on x and mean).
// ---------------------------------------------------------------------------------------------

fn translation_invariance<T>(native: f64, literal: f64)
where
    T: RealField + FromPrimitive,
{
    // The density depends on x and mu only through their difference, so shifting both by the same
    // constant leaves the log-density unchanged. Invariant, no closed form.
    let variance = lift::<T>(4.0);
    for shift in [-1000.0_f64, -7.5, 0.0, 3.25] {
        let base = gaussian_log_density(lift::<T>(2.0), lift::<T>(0.0), variance)
            .expect("variance 4 is positive");
        let shifted = gaussian_log_density(lift::<T>(2.0 + shift), lift::<T>(shift), variance)
            .expect("variance 4 is positive");
        assert_close(
            shifted,
            base,
            lift::<T>(native),
            "log-density is invariant under a shift of both x and the mean",
        );
    }

    // Anchored at an entirely negative location: x = -3, mu = -1, so x - mu = -2 and sigma^2 = 4.
    // Closed form, reusing the hand evaluation from test 3 (the deviation squared is again 4):
    //
    //   -1/2 * ln(2*pi*4) - (-2)^2 / (2*4)
    //     = -1.6120857137646180511975618578639 - 0.5
    //     = -2.1120857137646180511975618578639
    let expected = lift::<T>(-2.112_085_713_764_618);
    let got = gaussian_log_density(lift::<T>(-3.0), lift::<T>(-1.0), variance)
        .expect("variance 4 is positive");
    assert_close(
        got,
        expected,
        lift::<T>(literal),
        "log N(-3 | -1, sigma^2 = 4)",
    );
}

#[test]
fn test_translation_invariance_and_negative_locations() {
    translation_invariance::<f32>(F32.native, F32.literal);
    translation_invariance::<f64>(F64.native, F64.literal);
    translation_invariance::<Float106>(F106.native, F106.literal);
}
