/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{SampleSession, Uncertain};

const SEED: u64 = 0x5EED_2026;

#[test]
fn test_expected_value_zero_samples() {
    let session = SampleSession::seeded(SEED);
    let uncertain = Uncertain::<f64>::point(42.0);
    assert_eq!(uncertain.expected_value(&session, 0).unwrap(), 0.0);
}

#[test]
fn test_expected_value_single_sample() {
    let session = SampleSession::seeded(SEED);
    let uncertain = Uncertain::<f64>::point(42.0);
    assert_eq!(uncertain.expected_value(&session, 1).unwrap(), 42.0);
}

#[test]
fn test_expected_value_multiple_samples_constant() {
    let session = SampleSession::seeded(SEED);
    let uncertain = Uncertain::<f64>::point(42.0);
    assert_eq!(uncertain.expected_value(&session, 100).unwrap(), 42.0);
}

#[test]
fn test_expected_value_converges_to_the_closed_form() {
    // This test used to inject `0, 1, 2, 3, 4` through the cache and check that their mean was 2.
    // With no cache there is nothing to inject into, and that assertion was really exercising
    // `MeanAccumulator`, which `deep_causality_stats` tests. What belongs here is that this crate
    // reduces *its own draws* correctly, so the estimate is pinned against a distribution whose
    // mean is known in closed form.
    //
    // Uniform on [0, 10) has mean 5. Seeded, so the tolerance is a statement about the estimator
    // at this seed rather than a probabilistic hope.
    let session = SampleSession::seeded(SEED);
    let uncertain = Uncertain::uniform(0.0, 10.0);

    let mean = uncertain.expected_value(&session, 20_000).unwrap();

    assert!(
        (mean - 5.0).abs() < 0.1,
        "uniform(0, 10) has mean 5, estimator gave {mean}"
    );
}

#[test]
fn test_expected_value_is_the_same_answer_twice() {
    // The reducer holds its indices at `0..n`, so one session gives one estimate however many
    // times it is asked. A test can therefore assert on the number rather than around it.
    let session = SampleSession::seeded(SEED);
    let uncertain = Uncertain::normal(3.0, 1.0);

    let first = uncertain.expected_value(&session, 500).unwrap();
    let second = uncertain.expected_value(&session, 500).unwrap();

    assert_eq!(first, second);
}

#[test]
fn test_expected_value_differs_by_seed() {
    let uncertain = Uncertain::normal(3.0, 1.0);

    let left = uncertain
        .expected_value(&SampleSession::seeded(1), 500)
        .unwrap();
    let right = uncertain
        .expected_value(&SampleSession::seeded(2), 500)
        .unwrap();

    assert_ne!(left, right, "two seeds gave one estimate");
}

#[test]
fn test_standard_deviation_zero_samples() {
    let session = SampleSession::seeded(SEED);
    let uncertain = Uncertain::<f64>::point(42.0);
    assert_eq!(uncertain.standard_deviation(&session, 0).unwrap(), 0.0);
}

#[test]
fn test_standard_deviation_one_sample() {
    let session = SampleSession::seeded(SEED);
    let uncertain = Uncertain::<f64>::point(42.0);
    assert_eq!(uncertain.standard_deviation(&session, 1).unwrap(), 0.0);
}

#[test]
fn test_standard_deviation_multiple_samples_constant() {
    let session = SampleSession::seeded(SEED);
    let uncertain = Uncertain::<f64>::point(42.0);
    assert_eq!(uncertain.standard_deviation(&session, 100).unwrap(), 0.0);
}

#[test]
fn test_standard_deviation_converges_to_the_closed_form() {
    // As above: the injected `0..4` sequence was testing `std_dev`'s Bessel correction, which
    // belongs to the statistics crate. What this crate owes is that its own draws reduce to the
    // right spread.
    //
    // Uniform on [0, 10) has standard deviation `10 / sqrt(12)` ≈ 2.8868.
    let session = SampleSession::seeded(SEED);
    let uncertain = Uncertain::uniform(0.0, 10.0);

    let spread = uncertain.standard_deviation(&session, 20_000).unwrap();
    let expected = 10.0 / 12.0_f64.sqrt();

    assert!(
        (spread - expected).abs() < 0.1,
        "uniform(0, 10) has standard deviation {expected}, estimator gave {spread}"
    );
}

#[test]
fn test_standard_deviation_of_a_normal_recovers_its_scale() {
    // A second closed form, on a distribution whose spread is a parameter rather than derived,
    // so an error in the reducer cannot hide behind one family's algebra.
    let session = SampleSession::seeded(SEED);
    let uncertain = Uncertain::normal(0.0, 2.5);

    let spread = uncertain.standard_deviation(&session, 20_000).unwrap();

    assert!(
        (spread - 2.5).abs() < 0.1,
        "normal(0, 2.5) has standard deviation 2.5, estimator gave {spread}"
    );
}
