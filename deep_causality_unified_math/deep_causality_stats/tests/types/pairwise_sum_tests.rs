/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The balanced-tree accumulator every reduction in this crate sums through.
//!
//! # What this file is for
//!
//! A running total grows while its addends do not. Once the total passes the point where an addend
//! falls below its last place, every further addend rounds away — and the total stays finite while
//! it does, so there is no infinity and no error to notice, only a plausible number that is wrong.
//! Measured before the change, at `BFloat16` over draws from N(100, 5): a left-to-right mean read
//! 114.5 at 256 observations, 32.75 at a thousand and 3.28 at ten thousand, against a true 100.
//!
//! The type exists to make that impossible, so the cases below are chosen to *separate* a balanced
//! tree from a left-to-right fold rather than merely to exercise one. Any test whose sample is
//! short, or whose addends stay comparable to the total, passes under both and says nothing.
//!
//! The type is `pub(crate)`, so the two observations these cases need come through
//! `utils_tests::accumulation`: the total, and which slots hold a partial sum.
//!
//! # Provenance
//!
//! Every expected value is a closed form or an algebraic invariant, never a value recorded from a
//! run. Sums of `n` equal terms, Gauss's `n(n+1)/2`, and negation and cancellation invariants are
//! the whole vocabulary here, and each is exact at every scalar that can hold the result — which is
//! what lets the assertions be exact equality rather than a tolerance.

use deep_causality_num::{BFloat16, Float106, lift};
use deep_causality_stats::utils_tests::accumulation::{pairwise_occupancy, pairwise_total};

/// `n` copies of `value`, at the working scalar.
fn repeated<T: Copy>(value: T, n: usize) -> Vec<T> {
    vec![value; n]
}

/// `1..=n` at the working scalar.
fn ramp<T: deep_causality_num::FromPrimitive>(n: usize) -> Vec<T> {
    (1..=n).map(|i| lift::<T>(i as f64)).collect()
}

//
// Corner rows A, B and F — the counts and values where a fold can return its seed.
//

/// Row A. A sum over nothing is zero.
///
/// The additive identity is the answer here rather than a stand-in for one, which is what separates
/// this from the mean's empty case: `mean([])` is undefined and refuses, `Σ[]` is zero.
#[test]
fn test_a_sum_over_no_observations_is_zero() {
    let none: [f64; 0] = [];
    assert_eq!(pairwise_total(&none), 0.0);
    assert_eq!(
        pairwise_occupancy(&none),
        0,
        "no slot holds a partial sum before anything is pushed"
    );
}

/// Rows B and F. One observation comes back unchanged — including zero, a negative, and a negative
/// zero, where a total seeded with anything but `+0.0` would show.
#[test]
fn test_a_sum_over_one_observation_is_that_observation() {
    for v in [4.25_f64, 0.0, -9.5, 1e300, -1e-300] {
        assert_eq!(pairwise_total(&[v]), v, "the sum of [{v}] is {v}");
    }
    // `(+0) + (−0)` is `+0` under round-to-nearest while `(−0) + (−0)` is `−0`, so a total folded
    // from a zero seed turns every negative zero positive on the way out. The sign of zero is the
    // one thing a sum can lose without losing any magnitude, and IEEE 754 says what it should be,
    // so it is asserted rather than left to whichever seed the implementation happens to pick.
    assert!(
        pairwise_total(&[-0.0_f64]).is_sign_negative(),
        "a lone negative zero keeps its sign"
    );
    assert!(
        pairwise_total(&[-0.0_f64; 8]).is_sign_negative(),
        "(−0) + (−0) is −0, at a count that fills more than one slot"
    );
    assert!(
        pairwise_total(&[-0.0_f64, 0.0]).is_sign_positive(),
        "(−0) + (+0) is +0, which is the other half of the rule"
    );
}

//
// The arrangement itself — the invariant that makes two callers agree.
//

/// The documented invariant, checked at every count through the first three hundred: the set of
/// occupied slots is the binary representation of the number of observations.
///
/// This is the property that makes `mean` over a slice and `MeanAccumulator` over the same
/// sequence agree to the last bit. They cannot disagree about the association if both are decided
/// by the count alone, and the count is not something either of them chooses.
#[test]
fn test_the_occupied_slots_are_the_binary_representation_of_the_count() {
    let ones = repeated(1.0_f64, 300);
    for n in 0..=300usize {
        assert_eq!(
            pairwise_occupancy(&ones[..n]),
            n as u128,
            "after {n} observations the occupancy must be {n:b}"
        );
    }
}

/// Row D, where the association degenerates. A power of two makes one perfectly balanced tree and
/// its neighbours make the most lopsided arrangement the scheme produces — at `2ᵏ + 1` one slot
/// holds a single observation and another holds all the rest.
///
/// Provenance: the sum of `n` ones is `n`, exactly, at every count the scalar can hold.
#[test]
fn test_a_count_either_side_of_a_power_of_two_sums_exactly() {
    let ones = repeated(1.0_f64, 1100);
    for n in [
        1usize, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 256,
        257, 511, 512, 513, 1023, 1024, 1025,
    ] {
        assert_eq!(
            pairwise_total(&ones[..n]),
            n as f64,
            "the sum of {n} ones is {n}"
        );
    }
}

//
// Row G and the sign invariants.
//

/// Row G. Negating every observation negates the sum, whatever the association.
///
/// A sign dropped inside the tree, or applied twice, shows here rather than cancelling — and the
/// sample straddles zero so that neither side is the vanishing case.
#[test]
fn test_negating_every_observation_negates_the_sum() {
    let values: Vec<f64> = (1..=1000).map(|i| i as f64 * 0.5 - 250.0).collect();
    let negated: Vec<f64> = values.iter().map(|v| -v).collect();

    let positive_total = pairwise_total(&values);
    assert_ne!(
        positive_total, 0.0,
        "the fixture must not cancel, or the invariant is asserted where it vanishes"
    );
    assert_eq!(pairwise_total(&negated), -positive_total);
}

/// Row F at length. A sample that cancels term by term sums to exactly zero, and the companion
/// assertion — dropping one term must *not* still give zero — is what stops this from being the
/// vanishing case that passes for any implementation.
#[test]
fn test_a_cancelling_sample_sums_to_exactly_zero() {
    let mut alternating: Vec<f64> = Vec::new();
    for i in 1..=500 {
        alternating.push(i as f64);
        alternating.push(-(i as f64));
    }
    assert_eq!(pairwise_total(&alternating), 0.0, "±1…±500 cancels exactly");
    assert_eq!(
        pairwise_total(&alternating[..alternating.len() - 1]),
        500.0,
        "dropping the last term leaves exactly +500, not zero"
    );

    assert_eq!(pairwise_total(&repeated(0.0_f64, 1000)), 0.0);
}

/// Order independence where the exact sum is representable: the same multiset in ascending,
/// descending and interleaved order gives the same total.
///
/// Not true of floating-point summation in general — it is true here because these terms and every
/// partial sum of them are exact, which is the point. A left-to-right fold also passes this at
/// `f64`; it is at `BFloat16`, where the ascending order is the one that stalls, that the orders
/// part company under a naive fold.
#[test]
fn test_the_same_multiset_sums_the_same_in_any_order() {
    let ascending: Vec<BFloat16> = ramp(512);
    let mut descending = ascending.clone();
    descending.reverse();
    let mut interleaved: Vec<BFloat16> = Vec::new();
    for i in 0..256 {
        interleaved.push(ascending[i]);
        interleaved.push(ascending[511 - i]);
    }

    // Σ 1..=512 = 512·513/2 = 131328.
    let expected = lift::<BFloat16>(131_328.0);
    assert_eq!(pairwise_total(&ascending), expected, "ascending");
    assert_eq!(pairwise_total(&descending), expected, "descending");
    assert_eq!(pairwise_total(&interleaved), expected, "interleaved");
}

//
// Rows H, I and J — the extremes and what the type is allowed to do there.
//

/// Row I. A non-finite observation reaches the total rather than being skipped.
///
/// The reductions above decide what a `NaN` means for their statistic — `mean` keeps it, and says
/// so — and they cannot if the sum quietly drops it. The last case is the sharp one: `+∞` and `−∞`
/// together are a `NaN`, not a zero, and a fold that cancelled them would report a finite answer
/// for a sample that has none.
#[test]
fn test_a_non_finite_observation_reaches_the_total() {
    let mut with_nan = repeated(1.0_f64, 100);
    with_nan[50] = f64::NAN;
    assert!(pairwise_total(&with_nan).is_nan());

    let mut with_inf = repeated(1.0_f64, 100);
    with_inf[3] = f64::INFINITY;
    assert_eq!(pairwise_total(&with_inf), f64::INFINITY);

    assert!(
        pairwise_total(&[f64::INFINITY, f64::NEG_INFINITY]).is_nan(),
        "∞ − ∞ is NaN, not zero"
    );
}

/// Rows H and J. At the type's own extremes the sum saturates honestly.
///
/// An answer that leaves the type arrives as an infinity rather than as a plausible finite number,
/// which is exactly what lets `mean` and `dispersion` detect it and re-form through a scale. A
/// stalling fold is the failure this whole type exists to remove; saturating is not that failure,
/// it is the signal.
#[test]
fn test_a_sum_that_leaves_the_type_saturates_rather_than_stalling() {
    assert_eq!(pairwise_total(&[f64::MAX, f64::MAX]), f64::INFINITY);

    let tiny = f64::MIN_POSITIVE;
    assert_eq!(
        pairwise_total(&[tiny, tiny]),
        tiny + tiny,
        "nothing is lost on the way down"
    );

    // At the narrowest scalar the same holds at its own maximum, which is `f32`'s.
    let max = lift::<BFloat16>(3.389_531_389_251_535_5e38);
    assert!(
        !pairwise_total(&[max, max]).is_finite(),
        "2·MAX is not representable and must not come back as a number"
    );
}

//
// Row K — every scalar, and the case the whole type exists for.
//

/// Row K. A long reduction of addends that fall below the running total's last place is exact, at
/// every shipped scalar.
///
/// A thousand halves sum to 500. At `BFloat16` the spacing at 500 is 2, so an addend of 0.5
/// vanishes into a left-to-right total from about 256 onward and that total stalls there — a
/// plausible finite number roughly half the answer, with nothing to signal it. The tree never adds
/// a half to a hundred: it adds halves to halves and ones to ones, so every partial sum on the way
/// is exact and so is the result.
///
/// 500 is representable at `BFloat16` — the spacing there is 2 and 500 is even — so only the
/// arrangement can lose it. That is what makes this a test of the code and not of the format.
#[test]
fn test_a_thousand_addends_below_the_totals_last_place_sum_exactly_at_every_scalar() {
    assert_eq!(pairwise_total(&repeated(0.5_f64, 1000)), 500.0);
    assert_eq!(pairwise_total(&repeated(0.5_f32, 1000)), 500.0);
    assert_eq!(
        pairwise_total(&repeated(lift::<Float106>(0.5), 1000)),
        lift::<Float106>(500.0)
    );
    assert_eq!(
        pairwise_total(&repeated(lift::<BFloat16>(0.5), 1000)),
        lift::<BFloat16>(500.0),
        "500 is representable at BFloat16, so only the arrangement can lose it"
    );
}

/// The same at ten thousand, where a left-to-right `BFloat16` total is wrong by a factor of thirty
/// rather than of two.
///
/// Provenance: ten thousand copies of `1/64` sum to `156.25`, which is exact at every scalar here
/// (`1/64` and `156.25` are both dyadic and need seven significand bits).
#[test]
fn test_ten_thousand_addends_sum_exactly_at_the_narrowest_scalar() {
    let sixty_fourths = repeated(lift::<BFloat16>(1.0 / 64.0), 10_000);
    assert_eq!(
        pairwise_total(&sixty_fourths),
        lift::<BFloat16>(156.25),
        "10000/64 = 156.25"
    );
}

/// Row K on a count that is neither a power of two nor round, at a magnitude that is not dyadic.
///
/// Provenance: Gauss. `Σ 1..=n = n(n+1)/2`, so `Σ 1..=1000 = 500500` and `Σ 1..=997 = 497503`. 997
/// is prime, so its binary expansion is dense and nearly every slot carries a partial sum.
#[test]
fn test_a_ramp_sums_to_gauss_closed_form() {
    assert_eq!(pairwise_total(&ramp::<f64>(1000)), 500_500.0);
    assert_eq!(pairwise_total(&ramp::<f32>(1000)), 500_500.0);
    assert_eq!(pairwise_total(&ramp::<f64>(997)), 497_503.0);
    assert_eq!(
        pairwise_total(&ramp::<Float106>(997)),
        lift::<Float106>(497_503.0)
    );
}

/// Row C. A constant sample is where several arrangements coincide, so it is checked alongside a
/// varying one rather than alone — and at a magnitude where a naive total stalls.
///
/// Provenance: `n` copies of `c` sum to `n·c`. At `BFloat16`, 4096 copies of 8 sum to 32768, and
/// both the addend and the answer are exact powers of two.
#[test]
fn test_a_constant_sample_sums_to_the_count_times_the_constant() {
    let eights = repeated(lift::<BFloat16>(8.0), 4096);
    assert_eq!(pairwise_total(&eights), lift::<BFloat16>(32_768.0));

    // The varying companion, so the constant case is not the only evidence: a two-value sample
    // whose sum is exact but whose terms differ.
    let mut mixed: Vec<BFloat16> = Vec::new();
    for _ in 0..512 {
        mixed.push(lift::<BFloat16>(8.0));
        mixed.push(lift::<BFloat16>(24.0));
    }
    // 512·8 + 512·24 = 512·32 = 16384.
    assert_eq!(pairwise_total(&mixed), lift::<BFloat16>(16_384.0));
}
