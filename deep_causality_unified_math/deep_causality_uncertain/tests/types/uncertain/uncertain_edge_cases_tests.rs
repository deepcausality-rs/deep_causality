/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The edges of the real carrier: parameter ranges, degenerate counts, sign slips, error paths.
//!
//! Two rules from the crate's corner-case discipline apply throughout. **Never pin a quantity only
//! where it vanishes** — a mean of zero and a spread of zero cannot tell a working estimator from
//! one that returns its accumulator, so the assertions below sit away from the origin. And **never
//! let one input serve two rows** — each case names one property, so a failure says which.

use deep_causality_uncertain::{SampleSession, Uncertain, UncertainError};

const SEED: u64 = 0x5EED_2026;

// -------------------------------------------------------------------------------------------
// Distribution parameters: what is accepted, what is refused, and with which error.
// -------------------------------------------------------------------------------------------

/// A zero standard deviation is a **degenerate** normal, not an error: every draw is the mean.
#[test]
fn a_zero_standard_deviation_is_degenerate_rather_than_refused() {
    let session = SampleSession::seeded(SEED);
    let fixed = Uncertain::<f64>::normal(7.5, 0.0);

    for index in 0..16 {
        assert_eq!(fixed.sample_at(&session, index).expect("draw"), 7.5);
    }
}

/// A **negative** standard deviation is accepted, because it is the same symmetric distribution
/// reflected — `mean + sigma * z` with a negative sigma is `mean - sigma * z`, and `z` is symmetric.
///
/// Pinned because "sigma must be positive" is the intuition, and the contract is "sigma must be
/// finite". A reader who assumes the intuition would otherwise write a guard that changes nothing.
#[test]
fn a_negative_standard_deviation_is_a_reflection_not_an_error() {
    let session = SampleSession::seeded(SEED);
    let mirrored = Uncertain::<f64>::normal(0.0, -2.0);
    let upright = Uncertain::<f64>::normal(0.0, 2.0);

    let a: f64 = mirrored.expected_value(&session, 2000).expect("mean");
    let b: f64 = upright.expected_value(&session, 2000).expect("mean");

    assert!(a.is_finite() && b.is_finite());
    // The spread is the same magnitude either way.
    let sa: f64 = mirrored.standard_deviation(&session, 2000).expect("sd");
    let sb: f64 = upright.standard_deviation(&session, 2000).expect("sd");
    assert!(
        (sa - sb).abs() < 0.2,
        "a reflected normal must have the same spread: {sa} vs {sb}"
    );
}

/// A non-finite spread is refused, and the error names the distribution that refused it.
#[test]
fn a_non_finite_standard_deviation_is_refused() {
    let session = SampleSession::seeded(SEED);

    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let result = Uncertain::<f64>::normal(0.0, bad).sample_at(&session, 0);
        assert!(
            matches!(result, Err(UncertainError::NormalDistributionError(_))),
            "a standard deviation of {bad} must be refused, got {result:?}"
        );
    }
}

/// A non-finite **mean** is carried rather than refused: the mean is a value, and NaN in gives
/// NaN out, which is what IEEE arithmetic does everywhere else.
#[test]
fn a_non_finite_mean_is_carried_rather_than_refused() {
    let session = SampleSession::seeded(SEED);
    let drawn = Uncertain::<f64>::normal(f64::NAN, 1.0)
        .sample_at(&session, 0)
        .expect("a NaN mean is a value, not a refusal");
    assert!(drawn.is_nan());
}

/// A uniform range must be non-empty and finite, and each failure has its own error.
#[test]
fn uniform_refuses_an_empty_or_non_finite_range() {
    let session = SampleSession::seeded(SEED);

    // Reversed, and exactly-equal bounds, are both empty.
    for (low, high) in [(5.0, 1.0), (1.0, 1.0), (0.0, -0.0)] {
        let result = Uncertain::<f64>::uniform(low, high).sample_at(&session, 0);
        assert!(
            matches!(result, Err(UncertainError::UniformDistributionError(_))),
            "[{low}, {high}) must be refused as empty, got {result:?}"
        );
    }

    // A range that is non-empty but not finite is refused too.
    let result = Uncertain::<f64>::uniform(f64::NEG_INFINITY, f64::INFINITY).sample_at(&session, 0);
    assert!(matches!(
        result,
        Err(UncertainError::UniformDistributionError(_))
    ));
}

/// A uniform over a **negative** range draws inside it — the sign of the bounds is not assumed.
#[test]
fn a_uniform_over_a_negative_range_stays_inside_it() {
    let mut session = SampleSession::seeded(SEED);
    let u = Uncertain::<f64>::uniform(-10.0, -2.0);

    for v in u.take_samples(&mut session, 500).expect("draws") {
        assert!(
            (-10.0..-2.0).contains(&v),
            "draw {v} left the negative range [-10, -2)"
        );
    }
}

// -------------------------------------------------------------------------------------------
// Degenerate sample counts — the corner rows for every reducer.
// -------------------------------------------------------------------------------------------

/// Zero draws gives zero, and one draw gives zero spread, on every reducer.
///
/// These are the two counts where an estimator has nothing to estimate, and each returns the
/// scalar's zero rather than an error, because "no samples" is a legitimate request.
#[test]
fn the_degenerate_counts_return_zero_rather_than_failing() {
    let session = SampleSession::seeded(SEED);
    let q = Uncertain::<f64>::normal(100.0, 5.0);

    assert_eq!(q.expected_value(&session, 0).expect("n=0"), 0.0);
    assert_eq!(q.standard_deviation(&session, 0).expect("n=0"), 0.0);
    assert_eq!(q.standard_deviation(&session, 1).expect("n=1"), 0.0);
    assert_eq!(q.expected_value_qmc(0, 1).expect("qmc n=0"), 0.0);
    assert_eq!(q.standard_deviation_qmc(0, 1).expect("qmc n=0"), 0.0);
    assert_eq!(q.standard_deviation_qmc(1, 1).expect("qmc n=1"), 0.0);
    assert_eq!(
        q.estimate_probability_exceeds(&session, 100.0, 0)
            .expect("n=0"),
        0.0
    );
}

/// One draw has a mean — it is that draw — and it is **not** zero, which is what separates a
/// working estimator from one returning its accumulator.
#[test]
fn a_single_draw_has_that_draws_mean() {
    let session = SampleSession::seeded(SEED);
    let q = Uncertain::<f64>::normal(100.0, 5.0);

    let one: f64 = q.expected_value(&session, 1).expect("n=1");
    let drawn = q.sample_at(&session, 0).expect("draw");

    assert_eq!(one, drawn, "the mean of one draw is that draw");
    assert!(one > 50.0, "and it sits near 100, not at zero: {one}");
}

/// Two draws is the smallest count with a spread to estimate, and it is positive.
#[test]
fn two_draws_have_a_positive_spread() {
    let session = SampleSession::seeded(SEED);
    let q = Uncertain::<f64>::normal(100.0, 5.0);

    let sd: f64 = q.standard_deviation(&session, 2).expect("n=2");
    assert!(
        sd > 0.0,
        "two distinct draws must have a positive spread, got {sd}"
    );
}

// -------------------------------------------------------------------------------------------
// Sign slips.
// -------------------------------------------------------------------------------------------

/// A negative mean is recovered as a negative mean.
///
/// The whole suite would pass with a stray `abs()` if every quantity were positive.
#[test]
fn a_negative_mean_stays_negative() {
    let session = SampleSession::seeded(SEED);
    let q = Uncertain::<f64>::normal(-250.0, 4.0);

    let mean: f64 = q.expected_value(&session, 2000).expect("mean");
    assert!(
        (mean + 250.0).abs() < 5.0,
        "expected about -250, got {mean}"
    );
    assert!(mean < 0.0, "the sign must survive the estimator");
}

/// Negation flips every draw, and negating twice is the identity.
#[test]
fn negation_flips_each_draw_and_is_its_own_inverse() {
    let session = SampleSession::seeded(SEED);
    let q = Uncertain::<f64>::normal(3.0, 1.0);
    let negated = -q.clone();
    let twice = -(-q.clone());

    for index in 0..32 {
        let base = q.sample_at(&session, index).expect("draw");
        assert_eq!(negated.sample_at(&session, index).expect("draw"), -base);
        assert_eq!(twice.sample_at(&session, index).expect("draw"), base);
    }
}

/// Subtraction is not commutative, and `a - b` is the negation of `b - a` **only within one
/// graph**.
///
/// Across two separately built graphs it is not, and that is the ordinal scheme working rather
/// than failing: a leaf's ordinal is its position in *that graph's* traversal, so `a - b` gives `a`
/// ordinal 0 and `b` ordinal 1, while `b - a` gives them the opposite pair. Different addresses,
/// different draws. The trap is worth a test of its own because a reader will expect the pointwise
/// identity to hold.
#[test]
fn subtraction_is_not_commutative_and_only_negates_within_one_graph() {
    let session = SampleSession::seeded(SEED);
    let a = Uncertain::<f64>::normal(10.0, 1.0);
    let b = Uncertain::<f64>::normal(3.0, 1.0);

    let forward = a.clone() - b.clone();

    // Within one graph, negating the difference is the negation of every draw.
    let negated = -forward.clone();
    for index in 0..16 {
        let f = forward.sample_at(&session, index).expect("draw");
        assert_eq!(
            negated.sample_at(&session, index).expect("draw"),
            -f,
            "-(a - b) must be the negation of (a - b) at index {index}"
        );
        assert!(f > 0.0, "a - b should be positive here, got {f}");
    }

    // Built as a *separate* graph, `b - a` is a different address for each leaf and so is not the
    // pointwise negation. Pinned rather than assumed, and pinned as an inequality so that a change
    // making the ordinals graph-independent would be a visible decision.
    let backward = b - a;
    let f = forward.sample_at(&session, 0).expect("draw");
    let r = backward.sample_at(&session, 0).expect("draw");
    assert_ne!(
        f, -r,
        "two separately built graphs assign their own ordinals, so the draws differ"
    );
}

/// A leaf drawn at the same **ordinal** in two graphs agrees; at a different ordinal it does not.
///
/// This is the precise form of "two graphs sharing a leaf agree about that leaf". The agreement is
/// on the address, and the address includes the ordinal — so it holds when the traversal reaches
/// the leaf at the same position, and not otherwise.
#[test]
fn a_shared_leaf_agrees_exactly_when_its_ordinal_agrees() {
    let session = SampleSession::seeded(SEED);
    let x = Uncertain::<f64>::normal(0.0, 1.0);
    let y = Uncertain::<f64>::normal(1000.0, 1.0);

    // `x` is visited first in `x + y`, so it keeps ordinal 0 — the ordinal it has alone.
    let alone = x.sample_at(&session, 0).expect("draw");
    let sum = (x.clone() + y.clone())
        .sample_at(&session, 0)
        .expect("draw");
    let y_inside = sum - alone;
    assert!(
        (y_inside - 1000.0).abs() < 10.0,
        "x must contribute the same draw inside the sum as alone; y drew {y_inside}"
    );

    // `y` is visited second, so inside the sum it has ordinal 1 and draws differently from alone.
    let y_alone = y.sample_at(&session, 0).expect("draw");
    assert_ne!(
        y_inside, y_alone,
        "a leaf at a different ordinal is a different address, and draws differently"
    );
}

/// A comparison against a negative threshold is not a comparison against its magnitude.
#[test]
fn a_negative_threshold_is_not_its_magnitude() {
    let session = SampleSession::seeded(SEED);
    let q = Uncertain::<f64>::point(-5.0);

    assert!(
        q.greater_than(-10.0).sample_at(&session, 0).expect("draw"),
        "-5 > -10"
    );
    assert!(
        !q.greater_than(10.0).sample_at(&session, 0).expect("draw"),
        "-5 is not > 10"
    );
    assert!(
        q.less_than(0.0).sample_at(&session, 0).expect("draw"),
        "-5 < 0"
    );
}

/// A range that straddles zero is handled by both bounds, not by one.
#[test]
fn a_range_straddling_zero_tests_both_bounds() {
    let session = SampleSession::seeded(SEED);

    let inside = Uncertain::<f64>::point(0.0);
    let below = Uncertain::<f64>::point(-11.0);
    let above = Uncertain::<f64>::point(11.0);

    assert!(
        inside
            .within_range(-10.0, 10.0)
            .sample_at(&session, 0)
            .expect("draw")
    );
    assert!(
        !below
            .within_range(-10.0, 10.0)
            .sample_at(&session, 0)
            .expect("draw")
    );
    assert!(
        !above
            .within_range(-10.0, 10.0)
            .sample_at(&session, 0)
            .expect("draw")
    );
}

/// A reversed range is empty, and nothing is inside it.
#[test]
fn a_reversed_range_contains_nothing() {
    let session = SampleSession::seeded(SEED);
    for probe in [-1.0, 0.0, 3.0, 7.0] {
        assert!(
            !Uncertain::<f64>::point(probe)
                .within_range(5.0, 1.0)
                .sample_at(&session, 0)
                .expect("draw"),
            "{probe} must not be inside the reversed range [5, 1]"
        );
    }
}

/// `approx_eq` with a negative tolerance admits nothing — not even the exact target.
#[test]
fn approx_eq_with_a_negative_tolerance_admits_nothing() {
    let session = SampleSession::seeded(SEED);
    assert!(
        !Uncertain::<f64>::point(3.0)
            .approx_eq(3.0, -1.0)
            .sample_at(&session, 0)
            .expect("draw"),
        "a negative tolerance is an empty band, so even the target is outside it"
    );
    // A zero tolerance admits exactly the target.
    assert!(
        Uncertain::<f64>::point(3.0)
            .approx_eq(3.0, 0.0)
            .sample_at(&session, 0)
            .expect("draw")
    );
}

// -------------------------------------------------------------------------------------------
// The extremes of the scalar's range.
// -------------------------------------------------------------------------------------------

/// Arithmetic at the top of the range saturates to infinity rather than wrapping or panicking.
#[test]
fn arithmetic_at_the_top_of_the_range_saturates() {
    let session = SampleSession::seeded(SEED);
    let huge = Uncertain::<f64>::point(f64::MAX);

    let overflowed = (huge.clone() + huge.clone())
        .sample_at(&session, 0)
        .expect("draw");
    assert!(overflowed.is_infinite() && overflowed.is_sign_positive());

    let underflowed = (-huge.clone() - huge).sample_at(&session, 0).expect("draw");
    assert!(underflowed.is_infinite() && underflowed.is_sign_negative());
}

/// Division by a certain zero is infinity, and by a certain negative zero is negative infinity —
/// the sign of the zero decides, which is exactly the slip a normalising sum would hide.
#[test]
fn division_by_zero_takes_the_zeros_sign() {
    let session = SampleSession::seeded(SEED);
    let one = Uncertain::<f64>::point(1.0);

    let by_pos = (one.clone() / Uncertain::point(0.0))
        .sample_at(&session, 0)
        .expect("draw");
    let by_neg = (one / Uncertain::point(-0.0))
        .sample_at(&session, 0)
        .expect("draw");

    assert!(
        by_pos.is_infinite() && by_pos.is_sign_positive(),
        "1/+0 = +inf"
    );
    assert!(
        by_neg.is_infinite() && by_neg.is_sign_negative(),
        "1/-0 = -inf"
    );
}

/// The smallest positive normal survives a round trip through the graph.
#[test]
fn the_smallest_positive_normal_survives_a_round_trip() {
    let session = SampleSession::seeded(SEED);
    let tiny = Uncertain::<f64>::point(f64::MIN_POSITIVE);
    assert_eq!(
        tiny.sample_at(&session, 0).expect("draw"),
        f64::MIN_POSITIVE
    );
}

/// A `NaN` compares false against everything, including itself — so no verdict is silently true.
#[test]
fn nan_compares_false_against_everything() {
    let session = SampleSession::seeded(SEED);
    let nan = Uncertain::<f64>::point(f64::NAN);

    assert!(!nan.greater_than(0.0).sample_at(&session, 0).expect("draw"));
    assert!(!nan.less_than(0.0).sample_at(&session, 0).expect("draw"));
    assert!(!nan.equals(f64::NAN).sample_at(&session, 0).expect("draw"));
    assert!(
        !nan.within_range(f64::NEG_INFINITY, f64::INFINITY)
            .sample_at(&session, 0)
            .expect("draw")
    );
}

/// Infinity compares as itself, which distinguishes the `EqualTo` arm's infinity branch from its
/// epsilon branch.
#[test]
fn infinity_equals_itself_and_not_its_negation() {
    let session = SampleSession::seeded(SEED);
    let inf = Uncertain::<f64>::point(f64::INFINITY);

    assert!(
        inf.equals(f64::INFINITY)
            .sample_at(&session, 0)
            .expect("draw")
    );
    assert!(
        !inf.equals(f64::NEG_INFINITY)
            .sample_at(&session, 0)
            .expect("draw")
    );
    assert!(!inf.equals(0.0).sample_at(&session, 0).expect("draw"));
}

// -------------------------------------------------------------------------------------------
// `value()`, and the entropy delegations.
// -------------------------------------------------------------------------------------------

/// `value()` reads a certain root and reports zero for a graph that draws.
#[test]
fn value_reads_a_certain_root_and_reports_zero_otherwise() {
    assert_eq!(Uncertain::<f64>::point(-4.5).value(), -4.5);
    assert_eq!(Uncertain::<f64>::normal(100.0, 1.0).value(), 0.0);
    assert_eq!(Uncertain::<f64>::uniform(1.0, 2.0).value(), 0.0);
    assert_eq!(
        (Uncertain::<f64>::point(1.0) + Uncertain::point(2.0)).value(),
        0.0
    );
}

/// The `_from_entropy` forms delegate with their arguments in the right order.
///
/// They cannot be checked for a value — that is the point of them — so what is checked is that they
/// reach the same code path and produce something in range. An argument-order slip in the
/// delegation would show as a refusal or an out-of-range answer.
#[test]
fn the_entropy_delegations_reach_the_same_paths() {
    let q = Uncertain::<f64>::normal(50.0, 2.0);

    let mean = q.expected_value_from_entropy(500).expect("mean");
    assert!((mean - 50.0).abs() < 2.0, "entropy mean {mean} far from 50");

    let sd = q.standard_deviation_from_entropy(500).expect("sd");
    assert!((sd - 2.0).abs() < 1.0, "entropy sd {sd} far from 2");

    let one = q.sample_from_entropy().expect("draw");
    assert!(one.is_finite());

    let many = q.take_samples_from_entropy(32).expect("draws");
    assert_eq!(many.len(), 32);
    assert!(many.iter().all(|v| v.is_finite()));
}

/// `from_samples` summarises a slice, and its two degenerate inputs are defined rather than
/// refused.
#[test]
fn from_samples_handles_its_degenerate_inputs() {
    let session = SampleSession::seeded(SEED);

    // Empty: a point at zero.
    let empty = Uncertain::<f64>::from_samples(&[]);
    assert_eq!(empty.sample_at(&session, 0).expect("draw"), 0.0);

    // One sample: that value, with no spread.
    let single = Uncertain::<f64>::from_samples(&[7.0]);
    for index in 0..8 {
        assert_eq!(single.sample_at(&session, index).expect("draw"), 7.0);
    }

    // Many: the mean and spread of the slice, away from the origin so a stalled sum is visible.
    let data: Vec<f64> = (0..100).map(|i| 500.0 + i as f64).collect();
    let summarised = Uncertain::<f64>::from_samples(&data);
    let mean: f64 = summarised.expected_value(&session, 2000).expect("mean");
    assert!(
        (mean - 549.5).abs() < 5.0,
        "the summary's mean should be about 549.5, got {mean}"
    );
}

/// `estimate_probability_exceeds` counts strictly above the threshold, with the sign respected.
#[test]
fn estimate_probability_exceeds_counts_strictly_above() {
    let session = SampleSession::seeded(SEED);

    // Every draw is exactly the threshold, and "exceeds" is strict, so none of them count.
    let fixed = Uncertain::<f64>::point(-3.0);
    let at_threshold: f64 = fixed
        .estimate_probability_exceeds(&session, -3.0, 100)
        .expect("estimate");
    assert_eq!(
        at_threshold, 0.0,
        "strictly above means the threshold does not count"
    );

    let below_threshold: f64 = fixed
        .estimate_probability_exceeds(&session, -4.0, 100)
        .expect("estimate");
    assert_eq!(below_threshold, 1.0, "-3 exceeds -4 at every draw");
}
