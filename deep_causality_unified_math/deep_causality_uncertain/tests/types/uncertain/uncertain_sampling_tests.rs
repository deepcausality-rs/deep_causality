/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_uncertain::{SampleSession, Uncertain};

/// Every test below draws under this seed.
///
/// The seed arrives in the signature now rather than through a thread-local slot, so a test says
/// which stream it is asserting on instead of installing one and hoping nothing else displaced it.
const SEED: u64 = 0x5EED_2026;

//
// Tests for Uncertain<f64>
//

#[test]
fn test_f64_sample_at_point() {
    let session = SampleSession::seeded(SEED);
    let u = Uncertain::<f64>::point(42.0);
    assert_eq!(u.sample_at(&session, 0).unwrap(), 42.0);
}

#[test]
fn test_from_sample() {
    let session = SampleSession::seeded(SEED);
    let u = Uncertain::from_samples(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    assert!(u.sample_at(&session, 0).is_ok());
}

#[test]
fn test_from_sample_empty() {
    let session = SampleSession::seeded(SEED);
    // An empty sample is a point at zero — the degenerate answer this crate supplies where
    // `deep_causality_stats::mean` refuses. Asserting the value rather than `is_ok()`, which was
    // true for every possible sentinel and so said nothing about which one is returned.
    let u = Uncertain::from_samples(&[]);
    assert_eq!(u.sample_at(&session, 0).unwrap(), 0.0);
    assert_eq!(
        u.sample_at(&session, 7).unwrap(),
        0.0,
        "a point does not vary"
    );
}

#[test]
fn test_from_sample_single_observation_has_zero_spread() {
    let session = SampleSession::seeded(SEED);
    // One observation has no dispersion to estimate: `std_dev` says so with `InsufficientSamples`
    // and this crate answers zero, because a summary must summarise whatever it is handed.
    //
    // Pinned by *variation*, not by `is_ok()`: a normal with zero standard deviation returns its
    // mean at every index, so a non-zero sentinel would show up as two different draws.
    let u = Uncertain::from_samples(&[7.0]);
    assert_eq!(
        u.sample_at(&session, 0).unwrap(),
        7.0,
        "the single observation is the mean"
    );
    assert_eq!(
        u.sample_at(&session, 11).unwrap(),
        7.0,
        "zero spread means every draw is the mean"
    );
}

#[test]
fn test_f64_draw_is_stable_at_an_index() {
    // This asserted that a second draw came back from the cache. There is no cache; the value is
    // stable because it is a function of the seed, the index and the leaf's ordinal, which is the
    // property the cache was storing its way towards.
    let session = SampleSession::seeded(SEED);
    let u = Uncertain::uniform(0.0, 100.0);

    let first = u.sample_at(&session, 123).unwrap();
    let second = u.sample_at(&session, 123).unwrap();

    assert_eq!(first, second, "one address, one value");
    assert!((0.0..100.0).contains(&first), "and it lies in the support");
}

#[test]
fn test_f64_draw_differs_by_index() {
    // The other half of the same property: stability at an index would be worthless if every
    // index gave the same value.
    let session = SampleSession::seeded(SEED);
    let u = Uncertain::uniform(0.0, 100.0);

    assert_ne!(
        u.sample_at(&session, 123).unwrap(),
        u.sample_at(&session, 124).unwrap()
    );
}

#[test]
fn test_f64_sample_from_entropy() {
    let u = Uncertain::<f64>::point(7.0);
    // The seedless path: the index is not the caller's to choose, but a point distribution is the
    // same at every one.
    assert_eq!(u.sample_from_entropy().unwrap(), 7.0);
}

#[test]
fn test_f64_take_samples() {
    let mut session = SampleSession::seeded(SEED);
    let u = Uncertain::<f64>::point(88.0);
    let samples = u.take_samples(&mut session, 10).unwrap();
    assert_eq!(samples.len(), 10);
    assert!(samples.iter().all(|&s| s == 88.0));
    assert_eq!(
        session.position(),
        10,
        "the session advanced by one per draw"
    );
}

#[test]
fn test_f64_take_zero_samples() {
    let mut session = SampleSession::seeded(SEED);
    let u = Uncertain::<f64>::point(88.0);
    assert!(u.take_samples(&mut session, 0).unwrap().is_empty());
    assert_eq!(session.position(), 0, "no draw, no advance");
}

#[test]
fn test_estimate_probability_exceeds_normal() {
    let session = SampleSession::seeded(SEED);
    let u = Uncertain::normal(0.0, 1.0); // Standard normal distribution
    let num_samples = 10000;

    let prob = u
        .estimate_probability_exceeds(&session, 0.0, num_samples)
        .unwrap();
    // For a standard normal distribution, P(X > 0) should be close to 0.5
    assert!(
        (prob - 0.5).abs() < 0.05,
        "Expected probability near 0.5, got {}",
        prob
    );

    let prob_high = u
        .estimate_probability_exceeds(&session, 2.0, num_samples)
        .unwrap();
    // For a standard normal distribution, P(X > 2) should be small (approx 0.0228)
    assert!(
        prob_high < 0.05,
        "Expected probability less than 0.05, got {}",
        prob_high
    );
}

//
// Tests for Uncertain<bool>
//

#[test]
fn test_bool_sample_at_point() {
    let session = SampleSession::seeded(SEED);
    assert!(
        Uncertain::<bool>::point(true)
            .sample_at(&session, 0)
            .unwrap()
    );
    assert!(
        !Uncertain::<bool>::point(false)
            .sample_at(&session, 1)
            .unwrap()
    );
}

#[test]
fn test_bool_draw_is_stable_at_an_index() {
    let session = SampleSession::seeded(SEED);
    let u = Uncertain::bernoulli(0.5);

    let first = u.sample_at(&session, 456).unwrap();
    let second = u.sample_at(&session, 456).unwrap();

    assert_eq!(first, second, "one address, one value");
}

#[test]
fn test_bool_varies_across_indices() {
    // A fair coin that returned one face at every index would satisfy stability and be useless.
    let session = SampleSession::seeded(SEED);
    let u = Uncertain::bernoulli(0.5);
    let drawn: Vec<bool> = (0..64).map(|i| u.sample_at(&session, i).unwrap()).collect();

    assert!(drawn.iter().any(|&b| b) && drawn.iter().any(|&b| !b));
}

#[test]
fn test_bool_sample_from_entropy() {
    assert!(
        Uncertain::<bool>::point(true)
            .sample_from_entropy()
            .unwrap()
    );
}

#[test]
fn test_bool_take_samples() {
    let mut session = SampleSession::seeded(SEED);
    let u = Uncertain::<bool>::point(false);
    let samples = u.take_samples(&mut session, 20).unwrap();
    assert_eq!(samples.len(), 20);
    assert!(samples.iter().all(|&s| !s));
}

#[test]
fn test_bool_take_zero_samples() {
    let mut session = SampleSession::seeded(SEED);
    let u = Uncertain::<bool>::point(true);
    assert!(u.take_samples(&mut session, 0).unwrap().is_empty());
}
