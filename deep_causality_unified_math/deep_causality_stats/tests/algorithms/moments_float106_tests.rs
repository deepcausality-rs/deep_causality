/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Phase-2 suite for `mean`, `variance` and `std_dev` at `Float106`.
//!
//! One of three sibling files, one per shipped scalar: `moments_f32_tests.rs`,
//! `moments_f64_tests.rs` and `moments_float106_tests.rs`. `Float106` carries about thirty-two decimal digits, and the one case where `native` and `literal` diverge by ten orders. An expectation written as a decimal literal can only be checked to `F106.literal`; two quantities both computed in `Float106` must hold to `F106.native`.
//!
//! Written before the implementation, so every test here must fail on the
//! unimplemented panic. Every expected value is a hand-evaluated closed form written as
//! a literal with its arithmetic shown, an algebraic invariant, or the independent
//! pairwise oracle in `utils_tests::oracles` — never the implementation's formula
//! retyped, and never a value recorded from a run.
//!
//! Tolerances come from the one shared table in `utils_tests::precision`; this file
//! declares none of its own and reads the `F106` row.

use deep_causality_algebra::Real;
use deep_causality_num::Float106;
use deep_causality_num::lift;
use deep_causality_stats::utils_tests::assertions::{
    assert_close, assert_exact, assert_no_finite_answer, expect_empty_input,
    expect_insufficient_samples,
};
use deep_causality_stats::utils_tests::lift_array;
use deep_causality_stats::utils_tests::oracles::pairwise_variance_oracle;
use deep_causality_stats::utils_tests::precision::F106;
use deep_causality_stats::utils_tests::samples::FAMILY;
use deep_causality_stats::{mean, population_variance, std_dev, variance};

/// Provenance: closed form by hand. `2+4+4+4+5+5+7+9 = 40` over `n = 8`, so the mean is
/// `40/8 = 5`, written as the literal `5.0` and exact in binary at all three precisions.
///
/// The literal separates two near misses at once: the variance's divisor pasted into the mean would
/// give `40/7 ≈ 5.714`, and a sum that dropped the last element would give `31/7 ≈ 4.43`.
#[test]
fn test_mean_hand_computed() {
    let xs = lift_array::<Float106>(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
    let got = mean(&xs).expect("the mean of eight observations is defined");
    // 40/8 = 5.
    assert_close(
        got,
        lift::<Float106>(5.0),
        F106.native,
        "mean([2,4,4,4,5,5,7,9])",
    );
}

/// Corner rows G (negative) and F (zero).
///
/// Provenance: closed forms by hand. `(−3) + 1 + 5 = 3` over `n = 3` gives `3/3 = 1`; a `.abs()`
/// wrongly applied to the observations would give `9/3 = 3`. `(−2.5) + 2.5 = 0` over `n = 2` gives
/// exactly 0, and both halves and the result are exact in binary, so this one is asserted exactly.
#[test]
fn test_mean_over_negatives() {
    let xs = lift_array::<Float106>(&[-3.0, 1.0, 5.0]);
    let got = mean(&xs).expect("the mean of three observations is defined");
    // 3/3 = 1.
    assert_close(got, lift::<Float106>(1.0), F106.native, "mean([-3,1,5])");

    let cancels = lift_array::<Float106>(&[-2.5, 2.5]);
    let got = mean(&cancels).expect("the mean of two observations is defined");
    // 0/2 = 0, and the sample is not constant, so a zero here is not the vanishing case of a
    // constant sample. The non-zero mean above is the companion the "never pin a quantity only
    // where it vanishes" rule asks for.
    assert_exact(got, lift::<Float106>(0.0), "mean([-2.5, 2.5])");
}

/// Provenance: an algebraic invariant — the arithmetic mean is a convex combination of the sample,
/// so `min(xs) ≤ mean(xs) ≤ max(xs)` for every sample. The bounds are found by a scan that compares
/// and never accumulates, so it shares nothing with a sum divided by a count.
#[test]
fn test_mean_within_sample_range() {
    for sample in FAMILY {
        let xs = lift_array::<Float106>(sample);
        let mut lo = xs[0];
        let mut hi = xs[0];
        for &x in xs.iter().skip(1) {
            if x < lo {
                lo = x;
            }
            if x > hi {
                hi = x;
            }
        }
        let m = mean(&xs).expect("every family sample has at least two observations");
        assert!(
            lo <= m && m <= hi,
            "mean({sample:?}) = {:?} fell outside [{:?}, {:?}]",
            m.to_f64(),
            lo.to_f64(),
            hi.to_f64()
        );
    }
}

/// Corner rows A (empty), B (single element) and E (the documented threshold, both sides).
///
/// `mean`'s only threshold is `n ≥ 1`. Below it the doc is explicit: "the mean of no observations
/// is not zero, it is undefined", so `EmptyInput`. At it, the closed form is `x/1 = x` by hand, and
/// it is exact, so the observation must come back unchanged — including a negative one, where a
/// sum initialised to something other than zero would show.
#[test]
fn test_mean_threshold_zero_and_one_observation() {
    expect_empty_input(mean::<Float106>(&[]), "mean([])");

    let one = lift_array::<Float106>(&[4.25]);
    // 4.25/1 = 4.25.
    assert_exact(
        mean(&one).expect("the mean of one observation is that observation"),
        lift::<Float106>(4.25),
        "mean([4.25])",
    );

    let one_negative = lift_array::<Float106>(&[-9.5]);
    // -9.5/1 = -9.5.
    assert_exact(
        mean(&one_negative).expect("the mean of one observation is that observation"),
        lift::<Float106>(-9.5),
        "mean([-9.5])",
    );
}

/// Corner rows H (a value at the type's representable extreme) and J (overflow and underflow
/// reach, at the type's own extremes rather than at a fixed constant).
///
/// Provenance: closed forms by hand, each with an exactly representable answer.
///
/// * `mean([MAX, −MAX])`: the sum is `0` and `0/2 = 0`. The answer is representable, so an
///   intermediate that leaves the type is a defect, not a limitation — an incremental update of the
///   form `m ← m + (x − m)/k` computes `−MAX − MAX` here, which is `−∞`, and returns `−∞`.
/// * `mean([MIN_POSITIVE, MIN_POSITIVE])`: `2t/2 = t`, and `2t` is still normal, so nothing is lost
///   on the way down either.
/// * `mean([MAX])`: `MAX/1 = MAX`, the single-observation form at the top of the range.
#[test]
fn test_mean_at_type_extremes() {
    let max = lift::<Float106>(F106.max_finite);
    let straddling = lift_array::<Float106>(&[F106.max_finite, -F106.max_finite]);
    assert_exact(
        mean(&straddling).expect("the mean of two observations is defined"),
        lift::<Float106>(0.0),
        "mean([MAX, -MAX])",
    );

    let tiny = lift::<Float106>(F106.min_positive);
    let two_tiny = lift_array::<Float106>(&[F106.min_positive, F106.min_positive]);
    assert_exact(
        mean(&two_tiny).expect("the mean of two observations is defined"),
        tiny,
        "mean([MIN_POSITIVE, MIN_POSITIVE])",
    );

    let only_max = lift_array::<Float106>(&[F106.max_finite]);
    assert_exact(
        mean(&only_max).expect("the mean of one observation is that observation"),
        max,
        "mean([MAX])",
    );
}

/// Provenance: closed form by hand, on the sample whose arithmetic is checkable by eye.
///
/// ```text
/// xs   = [2, 4, 4, 4, 5, 5, 7, 9]        Σ = 40, n = 8,  x̄ = 40/8 = 5
/// x−x̄  = [−3, −1, −1, −1, 0, 0, 2, 4]
/// (x−x̄)² = [9, 1, 1, 1, 0, 0, 4, 16]      Σ = 32
/// variance = 32 / (n − 1) = 32 / 7
/// ```
///
/// `32/7 = 4.571428571428571428…`, written as `4.571_428_571_428_571` — the shortest decimal that
/// names the `f64` nearest the fraction, which is as close as an expected value written as a
/// literal can get (see [`Prec`]). The population `÷n` form the crate deliberately does not build
/// would give `32/8 = 4.0`, which this literal separates by 12%.
#[test]
fn test_variance_hand_computed() {
    let xs = lift_array::<Float106>(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
    let got = variance(&xs).expect("eight observations are enough for the corrected variance");
    // 32/7, hand-derived above.
    assert_close(
        got,
        lift::<Float106>(4.571_428_571_428_571),
        F106.literal,
        "variance([2,4,4,4,5,5,7,9])",
    );
}

/// The same closed form again, on a sample whose answer is exact in binary, so the assertion is not
/// limited by the decimal literal's own accuracy at `Float106`.
///
/// ```text
/// xs   = [1, 2, 3, 4, 5]                 Σ = 15, n = 5,  x̄ = 15/5 = 3
/// x−x̄  = [−2, −1, 0, 1, 2]
/// (x−x̄)² = [4, 1, 0, 1, 4]                Σ = 10
/// variance = 10 / (n − 1) = 10 / 4 = 2.5
/// ```
///
/// This is also corner row C at its sharpest for this function: the corrected `÷(n−1)` and the
/// population `÷n` forms coincide only where the sum of squares is zero, so a sample with non-zero
/// dispersion is the only place they can be told apart. Here they differ by 2.5 against 2.0.
#[test]
fn test_variance_exact_binary_sample() {
    let xs = lift_array::<Float106>(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    let got = variance(&xs).expect("five observations are enough for the corrected variance");
    // 10/4 = 2.5, hand-derived above; the population form would give 10/5 = 2.0.
    assert_close(
        got,
        lift::<Float106>(2.5),
        F106.native,
        "variance([1,2,3,4,5])",
    );
}

/// Provenance: the independent quadratic oracle, over the whole sample family.
///
/// One of the five family samples is constant, where both sides are zero; the other four have
/// non-zero dispersion, so the agreement is pinned somewhere the quantity does not vanish.
#[test]
fn test_variance_matches_pairwise_oracle() {
    for sample in FAMILY {
        let xs = lift_array::<Float106>(sample);
        let got = variance(&xs).expect("every family sample has at least two observations");
        let want = pairwise_variance_oracle(&xs);
        assert_close(
            got,
            want,
            F106.native,
            "variance against the pairwise oracle",
        );
    }
}

/// The two-element closed form, derived by hand and then checked over a family.
///
/// ```text
/// xs = [a, b]      x̄ = (a + b)/2
/// a − x̄ = (a − b)/2,  b − x̄ = (b − a)/2
/// Σ(x − x̄)² = 2·((b − a)/2)² = (b − a)²/2
/// variance  = (b − a)²/2 / (n − 1) = (b − a)²/2
/// ```
///
/// Corner row D, the degenerate index expression, lands here: at `n = 2` the divisor `n − 1` is 1,
/// so this case cannot separate an implementation that forgot to divide at all. That is what the
/// `n = 8` and `n = 5` cases above are for, and it is why this one is not the only value assertion.
#[test]
fn test_variance_two_element_closed_form() {
    let concrete = lift_array::<Float106>(&[1.0, 5.0]);
    // (5 − 1)²/2 = 16/2 = 8, exact in binary.
    assert_close(
        variance(&concrete).expect("two observations are enough"),
        lift::<Float106>(8.0),
        F106.native,
        "variance([1, 5])",
    );

    // The same closed form over a family, including a pair that straddles zero and a pair given in
    // descending order, where a `b − a` written as `a − b` would still square to the same thing but
    // a missing square would not.
    for &(a, b) in &[
        (1.0, 5.0),
        (-2.0, 6.0),
        (9.0, 3.0),
        (0.25, 0.75),
        (-1.5, -0.5),
    ] {
        let xs = lift_array::<Float106>(&[a, b]);
        let d = lift::<Float106>(b - a);
        let want = d * d / lift::<Float106>(2.0);
        assert_close(
            variance(&xs).expect("two observations are enough"),
            want,
            F106.native,
            "variance([a, b]) against (b − a)²/2",
        );
    }
}

/// Corner row B, and the deliberate behaviour change this crate carries.
///
/// The absorbed `variance_ddof1` returned `T::one()` for a sample shorter than two, and that
/// sentinel went on to feed a Gaussian density. The doc here refuses instead: "with one
/// observation, the divisor is zero and there is no dispersion to estimate". So any `Ok` fails this
/// case — the old sentinel most of all.
#[test]
fn test_variance_refuses_one_observation() {
    for &x in &[4.0, 0.0, -1.0, 1.0] {
        let xs = lift_array::<Float106>(&[x]);
        expect_insufficient_samples(variance(&xs), "variance([x]) on one observation");
    }
}

/// Corner row A. See reading 1 in the module header for why the empty sample is `EmptyInput` and
/// the one-element sample is `InsufficientSamples` rather than both being one variant.
#[test]
fn test_variance_refuses_empty_sample() {
    expect_empty_input(variance::<Float106>(&[]), "variance([])");
}

/// Corner rows C (two distinct quantities coincide — every observation equals the mean) and F.
///
/// Provenance: closed form by hand. Every deviation is `7 − 7 = 0`, so `Σ(x − x̄)² = 0` and
/// `0/3 = 0`. Nothing rounds, at any precision, so the zero is exact rather than merely small; a
/// result of `1e-16` would mean the deviations were not formed against the sample's own mean.
/// The negative constant is the same statement below zero.
#[test]
fn test_variance_of_constant_is_exactly_zero() {
    let xs = lift_array::<Float106>(&[7.0, 7.0, 7.0, 7.0]);
    assert_exact(
        variance(&xs).expect("four observations are enough"),
        lift::<Float106>(0.0),
        "variance([7,7,7,7])",
    );

    let negative = lift_array::<Float106>(&[-3.0, -3.0, -3.0]);
    assert_exact(
        variance(&negative).expect("three observations are enough"),
        lift::<Float106>(0.0),
        "variance([-3,-3,-3])",
    );
}

/// Provenance: an algebraic invariant — variance is translation invariant, `var(x + c) = var(x)`,
/// because adding `c` to every observation adds `c` to the mean and leaves every deviation alone.
///
/// The invariant is asserted against the hand-computed `32/7` rather than against a second call, so
/// the case pins a value as well as a symmetry. The largest shift, `c = 100`, puts the sample
/// around 105 where `Σxᵢ²` is ≈ 88 000 and the quantity to be recovered from it is 32; the
/// computational `Σxᵢ² − (Σxᵢ)²/n` form loses digits there that the deviation form does not.
#[test]
fn test_variance_translation_invariant() {
    let base = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    for &c in &[1.0, -10.0, 100.0, -0.5] {
        let shifted = lift_array::<Float106>(&base.map(|x| x + c));
        let got = variance(&shifted).expect("eight observations are enough");
        // 32/7 again, from the derivation on `case_variance_hand_computed`; the shift does not
        // change it.
        assert_close(
            got,
            lift::<Float106>(4.571_428_571_428_571),
            F106.literal,
            "variance of a translated sample",
        );
    }
}

/// The cancellation reach, at each type's own offset rather than at one shared constant.
///
/// Provenance: closed form by hand plus translation invariance. `[0, 1, 2]` has mean 1, squared
/// deviations `1 + 0 + 1 = 2`, and `2/(3 − 1) = 1`. Shifting all three by the offset leaves that
/// unchanged, so the answer is exactly 1 whatever the offset is.
///
/// At `f32` (offset 1e4) and `f64` (offset 1e8) the offset is large enough that `Σxᵢ²` carries the
/// answer in its last two or three bits, so the computational form returns 0 or 8 where the
/// deviation form returns 1. At `Float106` the offset is capped at 1e15 — not by the type, which
/// has 106 bits, but by the fixture's route through `f64`, where `1e16 + 1` is no longer
/// representable and the sample would silently collapse to a constant. So at that precision this
/// case asserts the value without separating the two forms, and the `f32` and `f64` runs are the
/// ones doing the separating.
#[test]
fn test_variance_offset_cancellation() {
    let off = F106.cancel_offset;
    let xs = lift_array::<Float106>(&[off, off + 1.0, off + 2.0]);
    let got = variance(&xs).expect("three observations are enough");
    // variance([0,1,2]) = 2/2 = 1, unchanged by the offset.
    assert_close(
        got,
        lift::<Float106>(1.0),
        F106.native,
        "variance of an offset ramp",
    );
}

/// Provenance: an algebraic invariant — variance is scale equivariant of degree two,
/// `var(k·x) = k²·var(x)`, because scaling every observation by `k` scales every deviation by `k`
/// and every squared deviation by `k²`. The base variance is the hand-computed `10/4 = 2.5` of
/// `[1, 2, 3, 4, 5]`.
///
/// A negative `k` is included (corner row G): the variance must come back positive, since `k²` is,
/// which a scale factor carried through without being squared would not.
#[test]
fn test_variance_scale_equivariant() {
    let base = lift_array::<Float106>(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    for &k in &[0.5, 2.0, 3.0, -2.0, -0.25] {
        let k = lift::<Float106>(k);
        let scaled: Vec<Float106> = base.iter().map(|&x| x * k).collect();
        // 2.5 is variance([1,2,3,4,5]) = 10/4, hand-derived on `case_variance_exact_binary_sample`.
        let want = k * k * lift::<Float106>(2.5);
        assert_close(
            variance(&scaled).expect("five observations are enough"),
            want,
            F106.native,
            "variance of a scaled sample against k²·2.5",
        );
    }
}

/// Corner row J, the overflow reach, at each type's own extreme.
///
/// The scale `k` is chosen per precision so that `Σxᵢ² = 55k²` leaves the type while
/// `Σ(xᵢ − x̄)² = 10k²` and the answer `2.5k²` stay inside it — 5e18 against `f32::MAX = 3.4e38`,
/// 2e153 against `f64::MAX = 1.8e308`, and the same 2e153 for `Float106`, whose head is an `f64`
/// and whose exponent range is therefore `f64`'s. The answer is representable, so `+∞` here is a
/// defect of the intermediate and not a limit of the type.
///
/// Provenance: scale equivariance again, over the hand-computed `10/4 = 2.5`.
///
/// The underflow direction is not a separate case: scaling the same sample down until `k²` reaches
/// the bottom of the type sends the true answer to zero as well, so both a correct implementation
/// and a broken one return zero and the input separates nothing.
#[test]
fn test_variance_overflow_reach() {
    let k = lift::<Float106>(F106.overflow_scale);
    let scaled: Vec<Float106> = lift_array::<Float106>(&[1.0, 2.0, 3.0, 4.0, 5.0])
        .iter()
        .map(|&x| x * k)
        .collect();
    let got = variance(&scaled).expect("five observations are enough");
    assert!(
        got.is_finite(),
        "variance at the type's overflow reach returned {:?}; the answer 2.5k² is representable",
        got.to_f64()
    );
    // 2.5 is variance([1,2,3,4,5]) = 10/4, scaled by k² under scale equivariance.
    assert_close(
        got,
        k * k * lift::<Float106>(2.5),
        F106.native,
        "variance at the type's overflow reach",
    );
}

/// Provenance: closed form by hand, chosen so the root is exact.
///
/// ```text
/// xs   = [1, 3, 5]                       Σ = 9, n = 3,  x̄ = 9/3 = 3
/// x−x̄  = [−2, 0, 2]
/// (x−x̄)² = [4, 0, 4]                      Σ = 8
/// variance = 8/(3 − 1) = 4,   std_dev = √4 = 2
/// ```
///
/// The population form would give `√(8/3) ≈ 1.633`, and a standard deviation that forgot the root
/// would give 4, so the literal `2.0` separates both.
#[test]
fn test_std_dev_hand_computed() {
    let xs = lift_array::<Float106>(&[1.0, 3.0, 5.0]);
    let got = std_dev(&xs).expect("three observations are enough");
    // √(8/2) = √4 = 2.
    assert_close(got, lift::<Float106>(2.0), F106.native, "std_dev([1,3,5])");
}

/// Provenance: an algebraic invariant, and the one the doc states outright — `std_dev` is "the
/// square root of `variance`", so squaring it must return the variance on every sample.
///
/// This is a round trip between two functions rather than a value pinned by one of them, and it is
/// the case that would catch a standard deviation quietly built on the population variance while
/// `variance` stayed corrected.
#[test]
fn test_std_dev_squared_is_variance() {
    for sample in FAMILY {
        let xs = lift_array::<Float106>(sample);
        let s = std_dev(&xs).expect("every family sample has at least two observations");
        let v = variance(&xs).expect("every family sample has at least two observations");
        assert_close(s * s, v, F106.native, "std_dev² against variance");
    }
}

/// Provenance: a bound that holds over the whole family — a square root of a non-negative quantity
/// is non-negative — plus the exact zero on the constant sample, where every deviation is zero and
/// `√0 = 0`. The family's other four samples have non-zero dispersion, so the zero is not the only
/// place the quantity is pinned.
#[test]
fn test_std_dev_non_negative_and_zero_on_constant() {
    let zero = lift::<Float106>(0.0);
    for sample in FAMILY {
        let xs = lift_array::<Float106>(sample);
        let s = std_dev(&xs).expect("every family sample has at least two observations");
        assert!(
            s >= zero,
            "std_dev({sample:?}) = {:?} is negative",
            s.to_f64()
        );
    }

    let constant = lift_array::<Float106>(&[7.0, 7.0, 7.0, 7.0]);
    assert_exact(
        std_dev(&constant).expect("four observations are enough"),
        zero,
        "std_dev([7,7,7,7])",
    );
}

/// Corner rows A, B and E for `std_dev`, which inherits `variance`'s threshold: it is the root of a
/// quantity that does not exist below two observations, so both refusals carry through.
///
/// The far side of the threshold is `n = 2`, where `variance([1, 5]) = (5 − 1)²/2 = 8` and the
/// standard deviation is `√8 = 2√2`. Provenance for the literal: `√2 = 1.41421356237309504880…`
/// (OEIS A002193), so `2√2 = 2.82842712474619009760…`.
#[test]
fn test_std_dev_thresholds() {
    expect_empty_input(std_dev::<Float106>(&[]), "std_dev([])");

    let one = lift_array::<Float106>(&[4.0]);
    expect_insufficient_samples(std_dev(&one), "std_dev([4]) on one observation");

    let two = lift_array::<Float106>(&[1.0, 5.0]);
    // 2·√2 = 2.8284271247461900976, from √2 = 1.4142135623730950488 (OEIS A002193).
    assert_close(
        std_dev(&two).expect("two observations are enough"),
        lift::<Float106>(2.8284271247461903),
        F106.literal,
        "std_dev([1, 5]) against 2√2",
    );
}

/// A `NaN` reaching each of the three functions. See reading 2 in the module header.
///
/// The sample is `[2, 4, 6, NaN]`, whose finite part has mean 4 and corrected variance 4 — so an
/// implementation that skipped the `NaN` would return a perfectly plausible number, which is the
/// defect this case exists to catch.
#[test]
fn test_non_finite_nan() {
    let xs = lift_array::<Float106>(&[2.0, 4.0, 6.0, f64::NAN]);
    assert_no_finite_answer(mean(&xs), "mean([2, 4, 6, NaN])");
    assert_no_finite_answer(variance(&xs), "variance([2, 4, 6, NaN])");
    assert_no_finite_answer(std_dev(&xs), "std_dev([2, 4, 6, NaN])");

    // A sample that is nothing but NaN, where a fold seeded with zero would return its seed.
    let all_nan = lift_array::<Float106>(&[f64::NAN, f64::NAN]);
    assert_no_finite_answer(mean(&all_nan), "mean([NaN, NaN])");
    assert_no_finite_answer(variance(&all_nan), "variance([NaN, NaN])");
}

/// Both infinities reaching each of the three functions.
///
/// The last sample is the sharp one: `[+∞, +∞]` is constant, and a constant-sample shortcut that
/// returns zero without looking at the values would report a dispersion of exactly zero for a
/// sample that has no finite mean to take deviations from. Under either reading of the docs that
/// is wrong — `∞ − ∞` is `NaN`, not 0.
#[test]
fn test_non_finite_infinities() {
    let with_pos = lift_array::<Float106>(&[1.0, f64::INFINITY]);
    assert_no_finite_answer(mean(&with_pos), "mean([1, +inf])");
    assert_no_finite_answer(variance(&with_pos), "variance([1, +inf])");
    assert_no_finite_answer(std_dev(&with_pos), "std_dev([1, +inf])");

    let with_neg = lift_array::<Float106>(&[1.0, f64::NEG_INFINITY]);
    assert_no_finite_answer(mean(&with_neg), "mean([1, -inf])");
    assert_no_finite_answer(variance(&with_neg), "variance([1, -inf])");

    let both = lift_array::<Float106>(&[f64::INFINITY, f64::NEG_INFINITY]);
    assert_no_finite_answer(mean(&both), "mean([+inf, -inf])");
    assert_no_finite_answer(variance(&both), "variance([+inf, -inf])");

    let constant_infinite = lift_array::<Float106>(&[f64::INFINITY, f64::INFINITY]);
    assert_no_finite_answer(mean(&constant_infinite), "mean([+inf, +inf])");
    assert_no_finite_answer(variance(&constant_infinite), "variance([+inf, +inf])");
    assert_no_finite_answer(std_dev(&constant_infinite), "std_dev([+inf, +inf])");
}

// -------------------------------------------------------------------------------------------
// Reach: an intermediate that leaves the type where the answer does not.
//
// The reductions below are the whole computation, not a step in it: `mean` sums and divides, and
// the variance sums squared deviations and divides. Each sum can leave the type on a sample whose
// answer is comfortably inside it, and an infinity returned there is a property of the
// intermediate rather than of the type.
// -------------------------------------------------------------------------------------------

/// Provenance: an algebraic invariant — the mean of a constant sample is that constant, for every
/// sample size. No arithmetic is retyped: the expectation is one of the inputs.
///
/// The sample is `[MAX, MAX]` at the type's own maximum, so the left-to-right sum is `2·MAX`,
/// which is `+∞` in every one of the four scalars, while the answer `MAX` is by construction the
/// largest value the type holds. `MAX` three times says the same thing where the sum saturates
/// before the last addend rather than at it.
#[test]
fn test_mean_of_a_constant_sample_at_the_type_maximum() {
    let max = lift::<Float106>(F106.max_finite);

    let two = [max, max];
    assert_exact(
        mean(&two).expect("the mean of two observations is defined"),
        max,
        "mean([MAX, MAX])",
    );

    let three = [max, max, max];
    assert_exact(
        mean(&three).expect("the mean of three observations is defined"),
        max,
        "mean([MAX, MAX, MAX])",
    );
}

/// Provenance: closed forms by hand, over a sample built from the type's own reach.
///
/// `v` is `0.8·√MAX`, so `v²` is `0.64·MAX` — inside the type — while `2v²` is `1.28·MAX`, which
/// is not. Both reductions below therefore overflow while forming their sum of squares, and both
/// answers are `v²`:
///
/// * `variance([−v, 0, v])`: the mean is `0`, the deviations are `[−v, 0, v]`, and
///   `Σd² / (n − 1) = (v² + 0 + v²) / 2 = v²`.
/// * `population_variance([−v, v])`: the mean is `0` again, and `Σd² / n = (v² + v²) / 2 = v²`.
///
/// The expectation is written as `v * v`, which is a product of one operand and not the
/// implementation's reduction; it is representable by the construction of `v`.
#[test]
fn test_variance_where_the_sum_of_squares_overflows_but_the_answer_does_not() {
    let v = lift::<Float106>(F106.max_finite).sqrt() * lift::<Float106>(0.8);
    let want = v * v;
    assert!(
        want.is_finite(),
        "the fixture is wrong: v² must be representable, got {:?}",
        want.to_f64()
    );
    assert!(
        !(want + want).is_finite(),
        "the fixture is wrong: 2v² must not be representable, got {:?}",
        (want + want).to_f64()
    );

    let symmetric = [-v, lift::<Float106>(0.0), v];
    let got = variance(&symmetric).expect("three observations are enough");
    assert!(
        got.is_finite(),
        "variance([-v, 0, v]) returned {:?}; the answer v² is representable",
        got.to_f64()
    );
    assert_close(got, want, F106.native, "variance([-v, 0, v]) is v²");

    let pair = [-v, v];
    let got = population_variance(&pair).expect("two observations are enough");
    assert!(
        got.is_finite(),
        "population_variance([-v, v]) returned {:?}; the answer v² is representable",
        got.to_f64()
    );
    assert_close(got, want, F106.native, "population_variance([-v, v]) is v²");
}
