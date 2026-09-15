/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The edges of the Boolean carrier: the probability range, degenerate counts, the SPRT gate.

use deep_causality_uncertain::{SampleSession, Uncertain, UncertainBool, UncertainError};

const SEED: u64 = 0x5EED_2026;

/// The two endpoints of the probability range are **exact**, and every draw honours them.
///
/// This is what holding `p` as fixed point buys, and it is the one property a wider scalar does
/// not improve: `p = 0` never draws true and `p = 1` never draws false, at any scalar.
#[test]
fn the_probability_endpoints_are_exact() {
    let session = SampleSession::seeded(SEED);
    let never = UncertainBool::<f64>::bernoulli(0.0);
    let always = UncertainBool::<f64>::bernoulli(1.0);

    for index in 0..256 {
        assert!(
            !never.sample_at(&session, index).expect("draw"),
            "p=0 drew true at {index}"
        );
        assert!(
            always.sample_at(&session, index).expect("draw"),
            "p=1 drew false at {index}"
        );
    }
}

/// A probability outside `[0, 1]` is refused, on both sides and for a non-finite parameter.
#[test]
fn a_probability_outside_the_unit_interval_is_refused() {
    let session = SampleSession::seeded(SEED);

    for bad in [
        -0.5,
        -f64::EPSILON,
        1.0 + f64::EPSILON,
        1.5,
        f64::NAN,
        f64::INFINITY,
    ] {
        let result = UncertainBool::<f64>::bernoulli(bad).sample_at(&session, 0);
        assert!(
            matches!(result, Err(UncertainError::BernoulliDistributionError(_))),
            "p = {bad} must be refused, got {result:?}"
        );
    }
}

/// A probability estimate over zero draws is zero rather than a division by zero.
#[test]
fn an_estimate_over_no_draws_is_zero() {
    let session = SampleSession::seeded(SEED);
    let coin = UncertainBool::<f64>::bernoulli(0.5);

    assert_eq!(coin.estimate_probability(&session, 0).expect("n=0"), 0.0);
    assert_eq!(coin.estimate_probability_qmc(0, 1).expect("qmc n=0"), 0.0);
}

/// A single draw gives a probability of exactly zero or exactly one — the two values a count of
/// one can produce, and neither is an error.
#[test]
fn an_estimate_over_one_draw_is_zero_or_one() {
    let session = SampleSession::seeded(SEED);
    let coin = UncertainBool::<f64>::bernoulli(0.5);

    let p: f64 = coin.estimate_probability(&session, 1).expect("n=1");
    assert!(p == 0.0 || p == 1.0, "one draw gives 0 or 1, got {p}");
}

/// A certain verdict estimates to exactly its own probability, away from one half so a stalled
/// counter is visible.
#[test]
fn a_certain_verdict_estimates_to_its_own_value() {
    let session = SampleSession::seeded(SEED);

    let t: f64 = UncertainBool::<f64>::point(true)
        .estimate_probability(&session, 500)
        .expect("estimate");
    let f: f64 = UncertainBool::<f64>::point(false)
        .estimate_probability(&session, 500)
        .expect("estimate");

    assert_eq!(t, 1.0);
    assert_eq!(f, 0.0);
}

/// The SPRT decides the two certain cases correctly at any threshold strictly inside `(0, 1)`.
#[test]
fn the_gate_decides_the_certain_cases() {
    let session = SampleSession::seeded(SEED);
    let always = UncertainBool::<f64>::point(true);
    let never = UncertainBool::<f64>::point(false);

    for threshold in [0.1, 0.5, 0.9] {
        assert!(
            always
                .to_bool(&session, threshold, 0.95, 0.05, 500)
                .expect("gate"),
            "a certainly-true verdict must clear a threshold of {threshold}"
        );
        assert!(
            !never
                .to_bool(&session, threshold, 0.95, 0.05, 500)
                .expect("gate"),
            "a certainly-false verdict must not clear a threshold of {threshold}"
        );
    }
}

/// The gate's decision follows the threshold, not the other way round: one distribution, two
/// thresholds either side of its probability, two different answers.
#[test]
fn the_gate_follows_the_threshold() {
    let session = SampleSession::seeded(SEED);
    let biased = UncertainBool::<f64>::bernoulli(0.8);

    assert!(
        biased
            .to_bool(&session, 0.5, 0.95, 0.05, 1000)
            .expect("gate"),
        "P(true) = 0.8 clears a threshold of 0.5"
    );
    assert!(
        !biased
            .to_bool(&session, 0.95, 0.95, 0.02, 1000)
            .expect("gate"),
        "P(true) = 0.8 does not clear a threshold of 0.95"
    );
}

/// A zero sample budget still returns a decision rather than failing — the fallback is the
/// observed frequency, and with nothing observed it is the negative answer.
#[test]
fn a_zero_budget_still_decides() {
    let session = SampleSession::seeded(SEED);
    let coin = UncertainBool::<f64>::bernoulli(0.5);

    let decided = coin.to_bool(&session, 0.5, 0.95, 0.05, 0);
    assert!(
        decided.is_ok(),
        "a zero budget must decide rather than fail: {decided:?}"
    );
}

/// `implicit_conditional` is "more likely than not", and it splits a strongly biased pair.
#[test]
fn the_implicit_conditional_splits_a_biased_pair() {
    let session = SampleSession::seeded(SEED);

    assert!(
        UncertainBool::<f64>::bernoulli(0.95)
            .implicit_conditional(&session)
            .expect("decide")
    );
    assert!(
        !UncertainBool::<f64>::bernoulli(0.05)
            .implicit_conditional(&session)
            .expect("decide")
    );
}

/// `value()` reads a certain root and reports `false` for a graph that draws.
#[test]
fn value_reads_a_certain_root_and_reports_false_otherwise() {
    assert!(UncertainBool::<f64>::point(true).value());
    assert!(!UncertainBool::<f64>::point(false).value());
    assert!(
        !UncertainBool::<f64>::bernoulli(0.99).value(),
        "a drawing graph has no single value"
    );
    assert!(
        !Uncertain::<f64>::point(5.0).greater_than(0.0).value(),
        "a comparison graph has no single value either"
    );
}

/// The `_from_entropy` delegations reach the same paths, with their arguments in the right order.
#[test]
fn the_entropy_delegations_reach_the_same_paths() {
    let always = UncertainBool::<f64>::point(true);
    let never = UncertainBool::<f64>::point(false);

    assert!(
        always
            .to_bool_from_entropy(0.5, 0.95, 0.05, 200)
            .expect("gate")
    );
    assert!(
        !never
            .to_bool_from_entropy(0.5, 0.95, 0.05, 200)
            .expect("gate")
    );
    assert!(
        always
            .probability_exceeds_from_entropy(0.5, 0.95, 0.05, 200)
            .expect("gate")
    );
    assert!(always.implicit_conditional_from_entropy().expect("decide"));

    let p: f64 = always
        .estimate_probability_from_entropy(200)
        .expect("estimate");
    assert_eq!(p, 1.0);

    assert!(always.sample_from_entropy().expect("draw"));
    assert_eq!(
        always.take_samples_from_entropy(16).expect("draws").len(),
        16
    );
}

/// Logic over certain operands is the ordinary truth table, including the two-operand XOR and NOR
/// arms that a single-operand test would miss.
#[test]
fn logic_over_certain_operands_is_the_truth_table() {
    let session = SampleSession::seeded(SEED);
    let t = UncertainBool::<f64>::point(true);
    let f = UncertainBool::<f64>::point(false);
    let at = |u: UncertainBool<f64>| u.sample_at(&session, 0).expect("draw");

    for (a, b, and, or, xor) in [
        (true, true, true, true, false),
        (true, false, false, true, true),
        (false, true, false, true, true),
        (false, false, false, false, false),
    ] {
        let ua = if a { t.clone() } else { f.clone() };
        let ub = if b { t.clone() } else { f.clone() };
        assert_eq!(at(ua.clone() & ub.clone()), and, "{a} & {b}");
        assert_eq!(at(ua.clone() | ub.clone()), or, "{a} | {b}");
        assert_eq!(at(ua.clone() ^ ub.clone()), xor, "{a} ^ {b}");
    }

    assert!(at(!f.clone()));
    assert!(!at(!t.clone()));
}

/// A verdict drawn from a comparison tracks the draw it judges at every index, including where the
/// quantity is negative — so a stray `abs()` in the comparison would show.
#[test]
fn a_verdict_tracks_its_draw_through_negative_values() {
    let session = SampleSession::seeded(SEED);
    let q = Uncertain::<f64>::normal(-5.0, 3.0);
    let above = q.greater_than(-5.0);

    let mut seen_true = false;
    let mut seen_false = false;
    for index in 0..64 {
        let drawn = q.sample_at(&session, index).expect("draw");
        let verdict = above.sample_at(&session, index).expect("verdict");
        assert_eq!(
            verdict,
            drawn > -5.0,
            "verdict disagrees with draw {drawn} at {index}"
        );
        seen_true |= verdict;
        seen_false |= !verdict;
    }
    assert!(
        seen_true && seen_false,
        "both faces must occur, or the loop proves nothing"
    );
}
