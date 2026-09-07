/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `log_sum_exp` and `log_add_exp`.
//!
//! # Where the expected values come from
//!
//! No expectation in this file is produced by the function under test, and none retypes the
//! stabilised `max + ln Σ exp(xᵢ − max)` formula. Every assertion draws on exactly one of:
//!
//! 1. **A closed form evaluated by hand.** `log Σ exp` over `n` equal entries collapses:
//!    `log(n·e^a) = a + log n`. The two instances used below are `n = 2`, giving `a + ln 2`,
//!    and `n = 4`, giving `a + ln 4 = a + 2 ln 2`. The arithmetic is written out at each call.
//! 2. **A published constant.** `ln 2` and `ln 4` are quoted to more digits than any of the
//!    three scalars can hold, with the source named at [`LN_2`] and [`LN_4`].
//! 3. **A different algorithm.** [`naive_log_sum_exp`] evaluates the definition directly —
//!    exponentiate, sum, take the log — with no maximum, no shift and no branch. It is the
//!    textbook form the crate's implementation exists to replace, so agreeing with it inside the
//!    range where it is well conditioned is a real cross-check, and *disagreeing* with it outside
//!    that range (where it returns `±∞`) is the property the crate is for.
//! 4. **Algebraic invariants.** Permutation invariance, shift equivariance
//!    (`lse(x + c) = lse(x) + c`), symmetry of the two-term form, the identity element `−∞`, the
//!    two-term form agreeing with the slice form, and the bracket `max x ≤ lse(x) ≤ max x + ln n`.
//!
//! # Why two tolerances per scalar
//!
//! [`Prec::native`] is the type's own working precision: it governs a comparison whose two sides
//! are both computed in `T`. [`Prec::decimal`] governs a comparison against an expectation that
//! reached `T` through an `f64` decimal literal. The fixtures lift from `f64`, and `Float106`'s
//! `From<f64>` fills only the high limb, so a lifted irrational carries `f64`'s ~1e-16 and not
//! `Float106`'s ~1e-31 — a `Float106` expectation built from a literal cannot be checked more
//! tightly than `f64` allows, whatever the result's own accuracy. Keeping the two apart is what
//! lets the invariant tests hold `Float106` to `Float106`'s precision.
//!
//! # Corner-case rows
//!
//! A empty ([`empty_slice_is_negative_infinity`]) · B single element
//! ([`single_element_returned_exactly`]) · C coincident quantities — every entry equal, so the
//! reduction is the `n = 2` and `n = 4` tie ([`all_equal_inputs_give_a_plus_log_of_the_count`],
//! [`log_add_exp_equal_arguments_give_a_plus_ln_2`]) · D degenerate index — the argmax is the only
//! index expression here, and it degenerates both when every entry ties for the maximum (row C) and
//! when the slice has one entry (row B); no other index arithmetic exists to degenerate · E
//! documented thresholds — the docs name no numeric threshold, only the finite/non-finite split on
//! the maximum, and both sides of it are covered ([`non_finite_maximum_saturates`] against every
//! finite-maximum test) · F zero ([`all_equal_inputs_give_a_plus_log_of_the_count`] at `a = 0`) ·
//! G negative ([`underflow_range_is_finite_and_exact`], and negative entries throughout) · H exact
//! domain boundary — the natural boundary for a log-domain reduction is a log-probability of
//! exactly 1 (`ln 1 = 0`) and exactly 0 (`ln 0 = −∞`), in
//! [`negative_infinity_entries_are_absorbed`] · I non-finite
//! ([`non_finite_maximum_saturates`]) · J the type's own extremes
//! ([`type_extremes_do_not_overflow`], at `1e38` for `f32` and `1e308` for the two wider scalars) ·
//! K every numeric property runs at `f32`, `f64` and `Float106` through one generic body, each
//! with its own tolerance.

use deep_causality_algebra::{Real, RealField};
use deep_causality_num::lift;
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_stats::utils_tests::lift_array;
use deep_causality_stats::utils_tests::precision::{F32, F64, F106, Prec};
use deep_causality_stats::{log_add_exp, log_sum_exp};

// ---------------------------------------------------------------------------------------------
// Published constants
// ---------------------------------------------------------------------------------------------

/// The natural logarithm of 2.
///
/// `ln 2 = 0.69314718055994530941723212145817656807550013436025525412…`
/// (OEIS A002162; Abramowitz & Stegun, *Handbook of Mathematical Functions*, Table 1.1.)
///
/// Those digits truncated to an `f64` are `0.6931471805599453`, and that is the value bound here.
/// It is spelled as the standard library's tabulated constant rather than retyped as that decimal
/// literal only because `clippy::approx_constant` denies the literal; it is the same published
/// number either way, and nothing under test produces it. The full digits are quoted above so a
/// reader can check the sums written at each call site by hand.
const LN_2: f64 = core::f64::consts::LN_2;

/// The natural logarithm of 4, which is `2 ln 2`.
///
/// `ln 4 = 1.38629436111989061883446424291635313615100026872051050824…`
/// (twice OEIS A002162.) Truncated to an `f64` that is `1.3862943611198906`, the literal below,
/// and because doubling is exact in binary floating point it is also bit-for-bit `2 · LN_2`.
const LN_4: f64 = 1.386_294_361_119_890_6;

// ---------------------------------------------------------------------------------------------
// Oracles and assertion helpers
// ---------------------------------------------------------------------------------------------

/// The definition, evaluated directly: `ln Σ exp(xᵢ)`.
///
/// A genuinely different algorithm from the one under test — it never forms a maximum, never
/// shifts, and has no branch on finiteness. It is well conditioned only while every `exp(xᵢ)` is
/// representable, which for the three scalars here means roughly `x ∈ [−87, 88]` for `f32` and
/// `x ∈ [−745, 709]` for `f64` and `Float106`. Inside that band it is the oracle; outside it, it
/// is the failure mode the crate's implementation exists to avoid, and the tests say so.
fn naive_log_sum_exp<T: Real + FromPrimitive>(values: &[T]) -> T {
    let mut sum = lift::<T>(0.0);
    for &v in values {
        sum += v.exp();
    }
    sum.ln()
}

/// `|actual − expected| ≤ rel · (1 + |expected|)`, a relative comparison with an absolute floor so
/// that an expectation of zero is still checkable.
fn assert_close<T: RealField + FromPrimitive>(actual: T, expected: T, rel: f64, what: &str) {
    let one = lift::<T>(1.0);
    let tol = lift::<T>(rel) * (one + expected.abs());
    // A NaN difference fails this comparison, which is what a NaN result deserves here.
    assert!((actual - expected).abs() <= tol, "{what}");
}

/// Asserts a bit-exact result. Used only where the closed form lands on a value every one of the
/// three scalars represents exactly, so no tolerance is warranted.
fn assert_exact<T: RealField + FromPrimitive>(actual: T, expected: T, what: &str) {
    assert!(actual == expected, "{what}");
}

fn zero<T: FromPrimitive>() -> T {
    lift::<T>(0.0)
}

// ---------------------------------------------------------------------------------------------
// 1. Agreement with the naive definition, inside the range where the naive form is sound
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 3. The expectation is [`naive_log_sum_exp`], a different algorithm.
fn agrees_with_naive_in_safe_range<T: RealField + FromPrimitive>(tol: Prec) {
    // Every entry is a dyadic rational, so the inputs themselves are exact at all three
    // precisions and the only error under test is the reduction's.
    let cases: [&[f64]; 6] = [
        &[0.0, 1.0, 2.0],
        &[-1.5, 0.25, 3.0, 0.0],
        &[2.0],
        &[-3.5, -3.0],
        &[0.5, 0.5, 0.5],
        &[4.0, -4.0],
    ];
    for case in cases {
        let xs = lift_array::<T>(case);
        let got = log_sum_exp(&xs);
        let want = naive_log_sum_exp(&xs);
        // The oracle is only an oracle while it is finite; assert that it is, so a silently
        // overflowing case can never masquerade as agreement.
        assert!(want.is_finite(), "naive oracle left its safe range");
        assert_close(
            got,
            want,
            tol.native,
            "log_sum_exp disagrees with ln(sum(exp(x)))",
        );
    }
}

#[test]
fn agrees_with_naive_in_safe_range_f32() {
    agrees_with_naive_in_safe_range::<f32>(F32);
}

#[test]
fn agrees_with_naive_in_safe_range_f64() {
    agrees_with_naive_in_safe_range::<f64>(F64);
}

#[test]
fn agrees_with_naive_in_safe_range_f106() {
    agrees_with_naive_in_safe_range::<Float106>(F106);
}

// ---------------------------------------------------------------------------------------------
// 2. Rows C, D and F: every entry equal, so the sum collapses to a count
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 1, a closed form evaluated by hand.
///
/// `log Σᵢ₌₁ⁿ exp(a) = log(n · e^a) = a + log n`.
///
/// At `a = 0` that is `log n` outright: two zeros give `ln 2 = 0.693147180559945309…` and four
/// zeros give `ln 4 = 1.386294361119890618…`. This is simultaneously row F (the input is zero),
/// row C (all `n` entries coincide, so the distribution is uniform and every entry ties for the
/// maximum) and row D (the argmax, the only index expression in the reduction, is therefore
/// ambiguous — any of the `n` indices is a correct maximiser and the answer must not depend on
/// which one is taken).
fn all_equal_inputs_give_a_plus_log_of_the_count<T: RealField + FromPrimitive>(tol: Prec) {
    // n = 2, a = 0: log(2 · e⁰) = log 2 = 0.6931471805599453094172321214581766
    let got = log_sum_exp(&lift_array::<T>(&[0.0, 0.0]));
    assert_close(
        got,
        lift::<T>(LN_2),
        tol.literal,
        "log_sum_exp([0, 0]) is not ln 2",
    );

    // n = 4, a = 0: log(4 · e⁰) = log 4 = 1.3862943611198906188344642429163531
    let got = log_sum_exp(&lift_array::<T>(&[0.0, 0.0, 0.0, 0.0]));
    assert_close(
        got,
        lift::<T>(LN_4),
        tol.literal,
        "log_sum_exp([0, 0, 0, 0]) is not ln 4",
    );

    // n = 2, a = 3: log(2 · e³) = 3 + ln 2 = 3.6931471805599453094172321214581766.
    // The expectation is assembled as 3 + ln 2 rather than written as one literal so that the
    // exactly representable part stays exact at every precision.
    let got = log_sum_exp(&lift_array::<T>(&[3.0, 3.0]));
    let want = lift::<T>(3.0) + lift::<T>(LN_2);
    assert_close(
        got,
        want,
        tol.literal,
        "log_sum_exp([3, 3]) is not 3 + ln 2",
    );

    // n = 2, a = -2.5: log(2 · e^-2.5) = -2.5 + ln 2 = -1.8068528194400546905827678785418234
    let got = log_sum_exp(&lift_array::<T>(&[-2.5, -2.5]));
    let want = lift::<T>(-2.5) + lift::<T>(LN_2);
    assert_close(
        got,
        want,
        tol.literal,
        "log_sum_exp([-2.5, -2.5]) is not -2.5 + ln 2",
    );
}

#[test]
fn all_equal_inputs_give_a_plus_log_of_the_count_f32() {
    all_equal_inputs_give_a_plus_log_of_the_count::<f32>(F32);
}

#[test]
fn all_equal_inputs_give_a_plus_log_of_the_count_f64() {
    all_equal_inputs_give_a_plus_log_of_the_count::<f64>(F64);
}

#[test]
fn all_equal_inputs_give_a_plus_log_of_the_count_f106() {
    all_equal_inputs_give_a_plus_log_of_the_count::<Float106>(F106);
}

// ---------------------------------------------------------------------------------------------
// 3. Row B: a single element
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 1. `log(e^a) = a` for every real `a`, with no rounding anywhere in
/// the identity, so the assertion is bit-exact rather than toleranced.
fn single_element_returned_exactly<T: RealField + FromPrimitive>(huge: f64) {
    // Dyadic rationals, exactly representable at all three precisions, plus the two magnitudes
    // where the naive form would have failed and the type's own extreme.
    for a in [0.0, 1.0, -3.25, 2.5, 1000.0, -800.0, huge, -huge] {
        let x = lift::<T>(a);
        let got = log_sum_exp(&[x]);
        assert_exact(got, x, "log_sum_exp([a]) is not a");
    }

    // The same identity at the boundary values, where "exactly" has to be read as "the same
    // non-finite value". `+∞` and `−∞` are their own log-sum-exp; `NaN` is checked by predicate
    // because `NaN != NaN`.
    let pos_inf = lift::<T>(f64::INFINITY);
    let got = log_sum_exp(&[pos_inf]);
    assert!(
        got.is_infinite() && got > zero::<T>(),
        "log_sum_exp([+inf]) is not +inf"
    );

    let neg_inf = lift::<T>(f64::NEG_INFINITY);
    let got = log_sum_exp(&[neg_inf]);
    assert!(
        got.is_infinite() && got < zero::<T>(),
        "log_sum_exp([-inf]) is not -inf"
    );

    let nan = lift::<T>(f64::NAN);
    let got = log_sum_exp(&[nan]);
    assert!(got.is_nan(), "log_sum_exp([NaN]) is not NaN");
}

#[test]
fn single_element_returned_exactly_f32() {
    single_element_returned_exactly::<f32>(F32.huge);
}

#[test]
fn single_element_returned_exactly_f64() {
    single_element_returned_exactly::<f64>(F64.huge);
}

#[test]
fn single_element_returned_exactly_f106() {
    single_element_returned_exactly::<Float106>(F106.huge);
}

// ---------------------------------------------------------------------------------------------
// 4. Row A: the empty slice
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 4, the identity element of the reduction, and the crate doc states
/// it outright: "The empty slice sums to zero and `log 0` is `−∞`, which is the identity for this
/// reduction." An identity is what makes `lse(xs ++ ys) = log_add_exp(lse(xs), lse(ys))` hold for
/// an empty `ys`, so the value is forced rather than chosen.
fn empty_slice_is_negative_infinity<T: RealField + FromPrimitive>() {
    let empty: [T; 0] = [];
    let got = log_sum_exp(&empty);
    assert!(got.is_infinite(), "log_sum_exp([]) is not infinite");
    assert!(got < zero::<T>(), "log_sum_exp([]) is not negative");

    // The identity law it stands for: adjoining the empty reduction changes nothing.
    let xs = lift_array::<T>(&[0.5, -1.5, 3.0]);
    let reduced = log_sum_exp(&xs);
    assert_exact(
        log_add_exp(reduced, got),
        reduced,
        "-inf is not the identity of log_add_exp",
    );
}

#[test]
fn empty_slice_is_negative_infinity_f32() {
    empty_slice_is_negative_infinity::<f32>();
}

#[test]
fn empty_slice_is_negative_infinity_f64() {
    empty_slice_is_negative_infinity::<f64>();
}

#[test]
fn empty_slice_is_negative_infinity_f106() {
    empty_slice_is_negative_infinity::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// 5. Where the naive form overflows
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 1 for the value, item 3 for the contrast.
///
/// `e^1000 ≈ 1.97e434` exceeds `f32::MAX` and `f64::MAX` alike, so `ln Σ exp` returns `+∞` at all
/// three precisions — the test asserts that, so the comparison has teeth. The stabilised reduction
/// is finite there, and its value is fixed by the same hand-evaluated collapse used above:
///
/// `log(2 · e^1000) = 1000 + ln 2 = 1000.6931471805599453094172321214581766`
/// `log(4 · e^1000) = 1000 + ln 4 = 1001.3862943611198906188344642429163531`
fn overflow_range_is_finite_and_exact<T: RealField + FromPrimitive>(tol: Prec) {
    let two = lift_array::<T>(&[1000.0, 1000.0]);
    let got = log_sum_exp(&two);
    assert!(got.is_finite(), "log_sum_exp([1000, 1000]) is not finite");
    assert_close(
        got,
        lift::<T>(1000.0) + lift::<T>(LN_2),
        tol.literal,
        "log_sum_exp([1000, 1000]) is not 1000 + ln 2",
    );
    assert!(
        !naive_log_sum_exp(&two).is_finite(),
        "the naive form was expected to overflow at 1000, so this case proves nothing"
    );

    let four = lift_array::<T>(&[1000.0, 1000.0, 1000.0, 1000.0]);
    let got = log_sum_exp(&four);
    assert!(got.is_finite(), "log_sum_exp([1000; 4]) is not finite");
    assert_close(
        got,
        lift::<T>(1000.0) + lift::<T>(LN_4),
        tol.literal,
        "log_sum_exp([1000; 4]) is not 1000 + ln 4",
    );

    // 800 is past `f32`'s and `f64`'s exp overflow alike: log(2 · e^800) = 800 + ln 2.
    let got = log_sum_exp(&lift_array::<T>(&[800.0, 800.0]));
    assert!(got.is_finite(), "log_sum_exp([800, 800]) is not finite");
    assert_close(
        got,
        lift::<T>(800.0) + lift::<T>(LN_2),
        tol.literal,
        "log_sum_exp([800, 800]) is not 800 + ln 2",
    );
}

#[test]
fn overflow_range_is_finite_and_exact_f32() {
    overflow_range_is_finite_and_exact::<f32>(F32);
}

#[test]
fn overflow_range_is_finite_and_exact_f64() {
    overflow_range_is_finite_and_exact::<f64>(F64);
}

#[test]
fn overflow_range_is_finite_and_exact_f106() {
    overflow_range_is_finite_and_exact::<Float106>(F106);
}

// ---------------------------------------------------------------------------------------------
// 6. Where the naive form underflows (row G, negative inputs)
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 1 for the value, item 3 for the contrast.
///
/// `e^-800 ≈ 2.6e-348` is below the smallest subnormal of `f32` and `f64` alike, so every term
/// flushes to zero, the sum is zero, and `ln 0 = −∞`. The stabilised reduction loses nothing:
///
/// `log(2 · e^-800) = -800 + ln 2 = -799.3068528194400546905827678785418234`
/// `log(4 · e^-800) = -800 + ln 4 = -798.6137056388801093811655357570836469`
fn underflow_range_is_finite_and_exact<T: RealField + FromPrimitive>(tol: Prec) {
    let two = lift_array::<T>(&[-800.0, -800.0]);
    let got = log_sum_exp(&two);
    assert!(got.is_finite(), "log_sum_exp([-800, -800]) is not finite");
    assert_close(
        got,
        lift::<T>(-800.0) + lift::<T>(LN_2),
        tol.literal,
        "log_sum_exp([-800, -800]) is not -800 + ln 2",
    );
    assert!(
        !naive_log_sum_exp(&two).is_finite(),
        "the naive form was expected to underflow at -800, so this case proves nothing"
    );

    let four = lift_array::<T>(&[-800.0, -800.0, -800.0, -800.0]);
    let got = log_sum_exp(&four);
    assert!(got.is_finite(), "log_sum_exp([-800; 4]) is not finite");
    assert_close(
        got,
        lift::<T>(-800.0) + lift::<T>(LN_4),
        tol.literal,
        "log_sum_exp([-800; 4]) is not -800 + ln 4",
    );
}

#[test]
fn underflow_range_is_finite_and_exact_f32() {
    underflow_range_is_finite_and_exact::<f32>(F32);
}

#[test]
fn underflow_range_is_finite_and_exact_f64() {
    underflow_range_is_finite_and_exact::<f64>(F64);
}

#[test]
fn underflow_range_is_finite_and_exact_f106() {
    underflow_range_is_finite_and_exact::<Float106>(F106);
}

// ---------------------------------------------------------------------------------------------
// 7. Underflow with distinct entries, checked against the oracle evaluated where it is sound
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 3 combined with item 4.
///
/// The entries `[-800, -799, -801]` all underflow, so the naive form cannot be applied to them
/// directly. It can be applied to `[0, 1, -1]`, which is the same input shifted by `+800` and sits
/// squarely in the well-conditioned band. Shift equivariance then transports the oracle's answer:
/// `lse([-800, -799, -801]) = lse([0, 1, -1]) - 800`. The expectation is therefore a different
/// algorithm's output plus an exactly representable shift, not the function under test.
fn underflow_matches_a_shifted_naive_oracle<T: RealField + FromPrimitive>(tol: Prec) {
    let shifted = lift_array::<T>(&[-800.0, -799.0, -801.0]);
    let got = log_sum_exp(&shifted);

    let centred = lift_array::<T>(&[0.0, 1.0, -1.0]);
    let oracle = naive_log_sum_exp(&centred);
    assert!(oracle.is_finite(), "naive oracle left its safe range");
    let want = oracle - lift::<T>(800.0);

    assert!(
        got.is_finite(),
        "log_sum_exp([-800, -799, -801]) is not finite"
    );
    assert_close(got, want, tol.native, "underflowing input lost accuracy");
}

#[test]
fn underflow_matches_a_shifted_naive_oracle_f32() {
    underflow_matches_a_shifted_naive_oracle::<f32>(F32);
}

#[test]
fn underflow_matches_a_shifted_naive_oracle_f64() {
    underflow_matches_a_shifted_naive_oracle::<f64>(F64);
}

#[test]
fn underflow_matches_a_shifted_naive_oracle_f106() {
    underflow_matches_a_shifted_naive_oracle::<Float106>(F106);
}

// ---------------------------------------------------------------------------------------------
// 8. Row I: a non-finite maximum
// ---------------------------------------------------------------------------------------------

/// Provenance: the documented rule, "A non-finite maximum saturates the result, because the shift
/// `x − max` is undefined there."
///
/// Reading taken here, stated because the doc leaves the word "saturates" to be interpreted: the
/// result *is* the non-finite maximum. `+∞` in gives `+∞` out, which is also the mathematical
/// answer since `Σ exp` diverges. All entries `−∞` gives `−∞` out, which is likewise the
/// mathematical answer (`log 0`) and the reading that keeps the empty-slice identity consistent —
/// a slice of identities must reduce to the identity. `NaN` propagates as `NaN`.
fn non_finite_maximum_saturates<T: RealField + FromPrimitive>() {
    // +inf is the maximum.
    let got = log_sum_exp(&lift_array::<T>(&[1.0, f64::INFINITY, 2.0]));
    assert!(
        got.is_infinite() && got > zero::<T>(),
        "a +inf entry did not saturate to +inf"
    );

    // +inf twice: still +inf, and no NaN from the undefined shift inf - inf.
    let got = log_sum_exp(&lift_array::<T>(&[f64::INFINITY, f64::INFINITY]));
    assert!(
        got.is_infinite() && got > zero::<T>(),
        "two +inf entries did not saturate to +inf"
    );

    // Every entry is -inf, so the maximum is -inf: the shift -inf - (-inf) is undefined and the
    // saturating rule returns the maximum itself.
    let got = log_sum_exp(&lift_array::<T>(&[f64::NEG_INFINITY, f64::NEG_INFINITY]));
    assert!(
        got.is_infinite() && got < zero::<T>(),
        "an all -inf slice did not saturate to -inf"
    );

    // NaN among finite entries.
    let got = log_sum_exp(&lift_array::<T>(&[1.0, f64::NAN, 2.0]));
    assert!(got.is_nan(), "a NaN entry did not propagate");

    // NaN beside an infinity.
    let got = log_sum_exp(&lift_array::<T>(&[f64::NAN, f64::INFINITY]));
    assert!(got.is_nan(), "NaN beside +inf did not propagate");
}

#[test]
fn non_finite_maximum_saturates_f32() {
    non_finite_maximum_saturates::<f32>();
}

#[test]
fn non_finite_maximum_saturates_f64() {
    non_finite_maximum_saturates::<f64>();
}

#[test]
fn non_finite_maximum_saturates_f106() {
    non_finite_maximum_saturates::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// 9. Row H: the exact domain boundary, a log-probability of exactly 0 and exactly 1
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 1, and item 4 for the identity law.
///
/// A log-domain reduction's boundary values are the logs of the probabilities 0 and 1, namely
/// `−∞` and `0`. An entry of `−∞` contributes `e^-∞ = 0` to the sum and must vanish:
///
/// `lse([-inf, 0]) = log(0 + e⁰) = log 1 = 0`, exactly.
/// `lse([-inf, 0, 0]) = log(0 + 1 + 1) = log 2 = 0.6931471805599453094172321214581766`
/// `lse([-inf, 4]) = log(0 + e⁴) = 4`, exactly.
///
/// The first and third land on values every scalar represents exactly, so they are asserted
/// bit-exact; the middle one is toleranced because `ln 2` is not.
fn negative_infinity_entries_are_absorbed<T: RealField + FromPrimitive>(tol: Prec) {
    let got = log_sum_exp(&lift_array::<T>(&[f64::NEG_INFINITY, 0.0]));
    assert_exact(got, zero::<T>(), "lse([-inf, 0]) is not 0");

    let got = log_sum_exp(&lift_array::<T>(&[f64::NEG_INFINITY, 4.0]));
    assert_exact(got, lift::<T>(4.0), "lse([-inf, 4]) is not 4");

    let got = log_sum_exp(&lift_array::<T>(&[f64::NEG_INFINITY, 0.0, 0.0]));
    assert_close(
        got,
        lift::<T>(LN_2),
        tol.literal,
        "lse([-inf, 0, 0]) is not ln 2",
    );

    // The same statement as an invariant: prepending the identity leaves the reduction alone.
    let xs = lift_array::<T>(&[0.5, -1.5, 3.0, 2.0]);
    let with_identity = lift_array::<T>(&[f64::NEG_INFINITY, 0.5, -1.5, 3.0, 2.0]);
    assert_close(
        log_sum_exp(&with_identity),
        log_sum_exp(&xs),
        tol.native,
        "a -inf entry changed the reduction",
    );
}

#[test]
fn negative_infinity_entries_are_absorbed_f32() {
    negative_infinity_entries_are_absorbed::<f32>(F32);
}

#[test]
fn negative_infinity_entries_are_absorbed_f64() {
    negative_infinity_entries_are_absorbed::<f64>(F64);
}

#[test]
fn negative_infinity_entries_are_absorbed_f106() {
    negative_infinity_entries_are_absorbed::<Float106>(F106);
}

// ---------------------------------------------------------------------------------------------
// 10. Permutation invariance
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 4. A sum over a set does not depend on the order of the set, so
/// neither does its log. Held to the type's own precision rather than to bit equality, because
/// reordering a floating-point summation legitimately changes the last place.
fn permutation_invariance<T: RealField + FromPrimitive>(tol: Prec) {
    let base = lift_array::<T>(&[0.5, -1.5, 3.0, 2.0, -1.0]);
    let want = log_sum_exp(&base);

    // A reversal, a rotation, and a permutation that moves the maximum off both ends.
    let permutations: [&[f64]; 3] = [
        &[-1.0, 2.0, 3.0, -1.5, 0.5],
        &[2.0, -1.0, 0.5, -1.5, 3.0],
        &[-1.5, 3.0, 0.5, -1.0, 2.0],
    ];
    for p in permutations {
        let got = log_sum_exp(&lift_array::<T>(p));
        assert_close(got, want, tol.native, "log_sum_exp depends on input order");
    }

    // The same, in the range where the naive form would have overflowed: reordering must not
    // disturb which entry is picked as the maximum.
    let base = lift_array::<T>(&[900.0, 901.0, 899.0]);
    let want = log_sum_exp(&base);
    for p in [&[901.0, 899.0, 900.0], &[899.0, 900.0, 901.0]] {
        let got = log_sum_exp(&lift_array::<T>(p));
        assert_close(
            got,
            want,
            tol.native,
            "log_sum_exp depends on input order in the overflow range",
        );
    }
}

#[test]
fn permutation_invariance_f32() {
    permutation_invariance::<f32>(F32);
}

#[test]
fn permutation_invariance_f64() {
    permutation_invariance::<f64>(F64);
}

#[test]
fn permutation_invariance_f106() {
    permutation_invariance::<Float106>(F106);
}

// ---------------------------------------------------------------------------------------------
// 11. Shift equivariance
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 4.
///
/// `log Σ exp(xᵢ + c) = log(e^c · Σ exp(xᵢ)) = c + log Σ exp(xᵢ)`, for every real `c`. This is the
/// identity the stabilised algorithm is built on, so it holding for shifts that carry the input
/// clean out of the naive form's range is the whole claim of the function.
fn shift_equivariance<T: RealField + FromPrimitive>(tol: Prec) {
    let raw = [0.5, -1.5, 3.0, 2.0];
    let xs = lift_array::<T>(&raw);
    let base = log_sum_exp(&xs);

    // c = 5 stays in range; c = ±700 leaves it for `f32` and brushes the edge for `f64`;
    // c = ±900 leaves it for all three.
    for c in [5.0, -5.0, 700.0, -700.0, 900.0, -900.0] {
        let moved: Vec<f64> = raw.iter().map(|x| x + c).collect();
        let got = log_sum_exp(&lift_array::<T>(&moved));
        let want = base + lift::<T>(c);
        assert!(got.is_finite(), "a shifted reduction is not finite");
        assert_close(
            got,
            want,
            tol.native,
            "log_sum_exp is not shift equivariant",
        );
    }
}

#[test]
fn shift_equivariance_f32() {
    shift_equivariance::<f32>(F32);
}

#[test]
fn shift_equivariance_f64() {
    shift_equivariance::<f64>(F64);
}

#[test]
fn shift_equivariance_f106() {
    shift_equivariance::<Float106>(F106);
}

// ---------------------------------------------------------------------------------------------
// 12. The bracket max x <= lse(x) <= max x + log n
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 4, a bound.
///
/// Every term of `Σ exp(xᵢ − max)` is in `(0, 1]` and one of them is exactly 1, so the sum lies in
/// `[1, n]` and its log in `[0, log n]`. With `n = 4`, `log n = ln 4 = 1.386294361119890618…`,
/// which is the published constant already quoted at [`LN_4`]. The bound is checked with a
/// tolerance slack so that a last-place rounding at either end is not read as a violation.
fn bracketed_by_max_and_max_plus_log_n<T: RealField + FromPrimitive>(tol: Prec) {
    let cases: [(&[f64], f64); 3] = [
        (&[0.5, -1.5, 3.0, 2.0], 3.0),
        (&[-4.0, -4.5, -6.0, -4.25], -4.0),
        (&[900.0, 899.0, 898.5, 897.0], 900.0),
    ];
    for (raw, max) in cases {
        let got = log_sum_exp(&lift_array::<T>(raw));
        let max_t = lift::<T>(max);
        let slack = lift::<T>(tol.native) * (lift::<T>(1.0) + max_t.abs());
        assert!(got.is_finite(), "a bracketed reduction is not finite");
        assert!(
            got >= max_t - slack,
            "log_sum_exp fell below the maximum entry"
        );
        assert!(
            got <= max_t + lift::<T>(LN_4) + slack,
            "log_sum_exp rose above max + ln 4 for a 4-entry slice"
        );
    }
}

#[test]
fn bracketed_by_max_and_max_plus_log_n_f32() {
    bracketed_by_max_and_max_plus_log_n::<f32>(F32);
}

#[test]
fn bracketed_by_max_and_max_plus_log_n_f64() {
    bracketed_by_max_and_max_plus_log_n::<f64>(F64);
}

#[test]
fn bracketed_by_max_and_max_plus_log_n_f106() {
    bracketed_by_max_and_max_plus_log_n::<Float106>(F106);
}

// ---------------------------------------------------------------------------------------------
// 13. Row J: the type's own extremes
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 4, a bound derived from the same hand-evaluated collapse.
///
/// `lse([a, a]) = a + ln 2`, and `0 < ln 2 < 1`, so the result must lie in `(a, a + 1)` and stay
/// finite — including when `a` is within a factor of a few of the type's largest representable
/// value, where any implementation that formed `e^a` or `2·e^a` would have overflowed. A bracket
/// rather than an equality, because at `1e308` the three scalars disagree about whether `a + ln 2`
/// is even distinguishable from `a`: `f32` and `f64` round it away, while `Float106`'s low limb
/// keeps it. Both behaviours satisfy the bracket, and both are correct.
fn type_extremes_do_not_overflow<T: RealField + FromPrimitive>(huge: f64) {
    let one = lift::<T>(1.0);

    for a in [huge, -huge] {
        let a_t = lift::<T>(a);
        assert!(a_t.is_finite(), "the chosen extreme is not representable");

        let got = log_sum_exp(&[a_t, a_t]);
        assert!(
            got.is_finite(),
            "log_sum_exp overflowed at the type's extreme"
        );
        assert!(
            got >= a_t,
            "log_sum_exp([a, a]) fell below a at the extreme"
        );
        assert!(
            got <= a_t + one,
            "log_sum_exp([a, a]) exceeded a + 1 at the extreme"
        );

        let got = log_add_exp(a_t, a_t);
        assert!(
            got.is_finite(),
            "log_add_exp overflowed at the type's extreme"
        );
        assert!(got >= a_t, "log_add_exp(a, a) fell below a at the extreme");
        assert!(
            got <= a_t + one,
            "log_add_exp(a, a) exceeded a + 1 at the extreme"
        );
    }

    // Mixed extremes: the smaller term underflows to nothing beside the larger, so the reduction
    // is the larger one exactly. log(e^huge + e^-huge) = huge + log(1 + e^(-2·huge)) = huge.
    let hi = lift::<T>(huge);
    let lo = lift::<T>(-huge);
    let got = log_sum_exp(&[hi, lo]);
    assert!(got.is_finite(), "a mixed-extreme reduction is not finite");
    assert_exact(got, hi, "log_sum_exp([huge, -huge]) is not huge");
}

#[test]
fn type_extremes_do_not_overflow_f32() {
    type_extremes_do_not_overflow::<f32>(F32.huge);
}

#[test]
fn type_extremes_do_not_overflow_f64() {
    type_extremes_do_not_overflow::<f64>(F64.huge);
}

#[test]
fn type_extremes_do_not_overflow_f106() {
    type_extremes_do_not_overflow::<Float106>(F106.huge);
}

// ---------------------------------------------------------------------------------------------
// 14. log_add_exp against the slice form
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 3. Two separate code paths — the doc says the two-term form exists
/// precisely so its caller need not build a slice — must compute the same quantity. Neither is the
/// expectation for the other's formula; they are independent implementations cross-checked.
fn log_add_exp_agrees_with_slice_form<T: RealField + FromPrimitive>(tol: Prec) {
    let pairs = [
        (0.0, 0.0),
        (1.0, 2.0),
        (-1.5, 3.0),
        (2.0, -7.0),
        (-4.5, -4.5),
        (1000.0, 1000.0),
        (1000.0, 999.0),
        (-800.0, -799.0),
        (-800.0, -800.0),
        (0.0, -50.0),
    ];
    for (a, b) in pairs {
        let (at, bt) = (lift::<T>(a), lift::<T>(b));
        let two_term = log_add_exp(at, bt);
        let slice_form = log_sum_exp(&[at, bt]);
        assert_close(
            two_term,
            slice_form,
            tol.native,
            "log_add_exp disagrees with log_sum_exp on the same two terms",
        );
    }
}

#[test]
fn log_add_exp_agrees_with_slice_form_f32() {
    log_add_exp_agrees_with_slice_form::<f32>(F32);
}

#[test]
fn log_add_exp_agrees_with_slice_form_f64() {
    log_add_exp_agrees_with_slice_form::<f64>(F64);
}

#[test]
fn log_add_exp_agrees_with_slice_form_f106() {
    log_add_exp_agrees_with_slice_form::<Float106>(F106);
}

// ---------------------------------------------------------------------------------------------
// 15. log_add_exp is symmetric
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 4. `e^a + e^b = e^b + e^a`, so the two-term reduction is symmetric.
/// Asserted bit-exact: unlike a reordered slice sum, swapping two addends changes nothing about
/// the floating-point operations performed, so any difference is a real asymmetry in the branch
/// that picks the larger term.
fn log_add_exp_is_symmetric<T: RealField + FromPrimitive>() {
    let pairs = [
        (0.0, 0.0),
        (1.0, 2.0),
        (-1.5, 3.0),
        (2.0, -7.0),
        (1000.0, 999.0),
        (-800.0, -801.0),
        (0.0, f64::NEG_INFINITY),
    ];
    for (a, b) in pairs {
        let (at, bt) = (lift::<T>(a), lift::<T>(b));
        assert_exact(
            log_add_exp(at, bt),
            log_add_exp(bt, at),
            "log_add_exp is not symmetric",
        );
    }

    // Symmetry at the non-finite arguments, where equality has to be read by predicate.
    let (inf, nan, one) = (
        lift::<T>(f64::INFINITY),
        lift::<T>(f64::NAN),
        lift::<T>(1.0),
    );
    assert!(
        log_add_exp(inf, one).is_infinite() && log_add_exp(one, inf).is_infinite(),
        "log_add_exp is not symmetric at +inf"
    );
    assert!(
        log_add_exp(nan, one).is_nan() && log_add_exp(one, nan).is_nan(),
        "log_add_exp is not symmetric at NaN"
    );
}

#[test]
fn log_add_exp_is_symmetric_f32() {
    log_add_exp_is_symmetric::<f32>();
}

#[test]
fn log_add_exp_is_symmetric_f64() {
    log_add_exp_is_symmetric::<f64>();
}

#[test]
fn log_add_exp_is_symmetric_f106() {
    log_add_exp_is_symmetric::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// 16. Rows C and D for the two-term form: equal arguments
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 1.
///
/// `log(e^a + e^a) = log(2 e^a) = a + ln 2`, for every real `a`. When the two arguments coincide
/// the "pick the larger" step has no unique answer, which is this family's row-D degeneracy, and
/// the result must not depend on how the tie is broken. Checked at `a = 0`, where the answer is
/// `ln 2` outright; at `a = 1000`, where `e^a` overflows every one of the three scalars; and at
/// `a = -800`, where `e^a` flushes to zero in every one of them.
fn log_add_exp_equal_arguments_give_a_plus_ln_2<T: RealField + FromPrimitive>(tol: Prec) {
    // a = 0: log 2 = 0.6931471805599453094172321214581766
    let got = log_add_exp(zero::<T>(), zero::<T>());
    assert_close(
        got,
        lift::<T>(LN_2),
        tol.literal,
        "log_add_exp(0, 0) is not ln 2",
    );

    for a in [1.0, -2.5, 1000.0, -800.0, 800.0] {
        let at = lift::<T>(a);
        let got = log_add_exp(at, at);
        assert!(got.is_finite(), "log_add_exp(a, a) is not finite");
        assert_close(
            got,
            at + lift::<T>(LN_2),
            tol.literal,
            "log_add_exp(a, a) is not a + ln 2",
        );
    }
}

#[test]
fn log_add_exp_equal_arguments_give_a_plus_ln_2_f32() {
    log_add_exp_equal_arguments_give_a_plus_ln_2::<f32>(F32);
}

#[test]
fn log_add_exp_equal_arguments_give_a_plus_ln_2_f64() {
    log_add_exp_equal_arguments_give_a_plus_ln_2::<f64>(F64);
}

#[test]
fn log_add_exp_equal_arguments_give_a_plus_ln_2_f106() {
    log_add_exp_equal_arguments_give_a_plus_ln_2::<Float106>(F106);
}

// ---------------------------------------------------------------------------------------------
// 17. log_add_exp at the identity and at the non-finite arguments (rows H and I)
// ---------------------------------------------------------------------------------------------

/// Provenance: allow-list item 1 for the identity, and the documented saturation rule for the
/// rest.
///
/// `log(e^-∞ + e^a) = log(0 + e^a) = a`, exactly, for every finite `a`: `−∞` is the identity of
/// this operation, the same identity the empty slice reduces to. `+∞` as the larger argument
/// saturates the result, as does `NaN`. Two `−∞` arguments have a non-finite maximum and saturate
/// to `−∞`, which is also the mathematical answer.
fn log_add_exp_identity_and_non_finite<T: RealField + FromPrimitive>() {
    let neg_inf = lift::<T>(f64::NEG_INFINITY);
    let pos_inf = lift::<T>(f64::INFINITY);
    let nan = lift::<T>(f64::NAN);

    for a in [0.0, 1.0, -3.25, 1000.0, -800.0] {
        let at = lift::<T>(a);
        assert_exact(
            log_add_exp(neg_inf, at),
            at,
            "-inf is not the left identity of log_add_exp",
        );
        assert_exact(
            log_add_exp(at, neg_inf),
            at,
            "-inf is not the right identity of log_add_exp",
        );
    }

    let got = log_add_exp(neg_inf, neg_inf);
    assert!(
        got.is_infinite() && got < zero::<T>(),
        "log_add_exp(-inf, -inf) is not -inf"
    );

    let got = log_add_exp(pos_inf, lift::<T>(3.0));
    assert!(
        got.is_infinite() && got > zero::<T>(),
        "log_add_exp(+inf, a) is not +inf"
    );

    let got = log_add_exp(pos_inf, pos_inf);
    assert!(
        got.is_infinite() && got > zero::<T>(),
        "log_add_exp(+inf, +inf) is not +inf"
    );

    assert!(
        log_add_exp(nan, lift::<T>(3.0)).is_nan(),
        "log_add_exp(NaN, a) is not NaN"
    );
    assert!(
        log_add_exp(nan, nan).is_nan(),
        "log_add_exp(NaN, NaN) is not NaN"
    );
}

#[test]
fn log_add_exp_identity_and_non_finite_f32() {
    log_add_exp_identity_and_non_finite::<f32>();
}

#[test]
fn log_add_exp_identity_and_non_finite_f64() {
    log_add_exp_identity_and_non_finite::<f64>();
}

#[test]
fn log_add_exp_identity_and_non_finite_f106() {
    log_add_exp_identity_and_non_finite::<Float106>();
}
