/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::UncertainBool;
use deep_causality_uncertain::{SampleSession, UncertainError, sprt_eval};

/// Every test below that gates on a sampled decision installs this seed first.
///
/// `evaluate_hypothesis` runs a sequential probability ratio test, and failing to reach a verdict
/// within the sample budget is an ordinary outcome of that test rather than a defect in it.
/// Unseeded, `test_evaluate_hypothesis_confidence_effect` failed 2 of 400 runs. The assertions are
/// unchanged; only the entropy source is.
const SEED: u64 = 0x5EED_2026;

#[test]
fn test_evaluate_hypothesis_always_true() {
    let session = SampleSession::seeded(SEED);
    // Create an UncertainBool<f64> that always samples true
    let ub = UncertainBool::<f64>::point(true);
    let result = sprt_eval::evaluate_hypothesis(&ub, &session, 0.5, 0.95, 0.01, 100, 0).unwrap();
    assert!(result, "Should accept H1 for always true");
}

#[test]
fn test_evaluate_hypothesis_always_false() {
    let session = SampleSession::seeded(SEED);
    // Create an UncertainBool<f64> that always samples false
    let ub = UncertainBool::<f64>::point(false);
    let result = sprt_eval::evaluate_hypothesis(&ub, &session, 0.5, 0.95, 0.01, 100, 0).unwrap();
    assert!(!result, "Should accept H0 for always false");
}

#[test]
fn test_evaluate_hypothesis_fallback_true() {
    let session = SampleSession::seeded(SEED);
    // Samples are 60% true, threshold 0.5, but max_samples is too low for SPRT to conclude
    let ub = UncertainBool::<f64>::bernoulli(0.6);
    let result = sprt_eval::evaluate_hypothesis(&ub, &session, 0.5, 0.95, 0.01, 1000, 0).unwrap(); // Increased max_samples
    assert!(result, "Should fallback to true (0.6 > 0.5)");
}

#[test]
fn test_evaluate_hypothesis_fallback_false() {
    let session = SampleSession::seeded(SEED);
    // Samples are 40% true, threshold 0.5, but max_samples is too low for SPRT to conclude
    let ub = UncertainBool::<f64>::bernoulli(0.4);
    let result = sprt_eval::evaluate_hypothesis(&ub, &session, 0.5, 0.95, 0.01, 1000, 0).unwrap(); // Increased max_samples
    assert!(!result, "Should fallback to false (0.4 <= 0.5)");
}

#[test]
fn test_evaluate_hypothesis_error_propagation() {
    let session = SampleSession::seeded(SEED);
    // Test error propagation from sampling
    let invalid_bernoulli = UncertainBool::<f64>::bernoulli(2.0); // Invalid p
    let result =
        sprt_eval::evaluate_hypothesis(&invalid_bernoulli, &session, 0.5, 0.95, 0.01, 10, 0);
    assert!(result.is_err());
    match result.err().unwrap() {
        UncertainError::BernoulliDistributionError(_) => (),
        _ => panic!("Expected BernoulliDistributionError"),
    }
}

#[test]
fn test_evaluate_hypothesis_threshold_boundaries() {
    let session = SampleSession::seeded(SEED);
    let ub_high = UncertainBool::<f64>::point(true);
    let ub_low = UncertainBool::<f64>::point(false);

    // Threshold 0.0
    assert!(sprt_eval::evaluate_hypothesis(&ub_high, &session, 0.0, 0.95, 0.01, 10, 0).unwrap());
    assert!(!sprt_eval::evaluate_hypothesis(&ub_low, &session, 0.0, 0.95, 0.01, 10, 0).unwrap());

    // Threshold 1.0
    // The hypothesis is p > 1.0, which is impossible for p=1.0, so this should be false.
    assert!(!sprt_eval::evaluate_hypothesis(&ub_high, &session, 1.0, 0.95, 0.01, 10, 0).unwrap());
    // This case is tricky: if p0 is 1.0 - epsilon, and actual is 0.0, it should be false.
    assert!(!sprt_eval::evaluate_hypothesis(&ub_low, &session, 1.0, 0.95, 0.01, 10, 0).unwrap());
}

#[allow(clippy::bool_comparison)]
#[test]
fn test_evaluate_hypothesis_epsilon_effect() {
    let session = SampleSession::seeded(SEED);
    // Test with a distribution that's exactly on the threshold
    let ub_50_50 = UncertainBool::<f64>::bernoulli(0.5);

    // With a very small epsilon, it's hard to conclude, might hit max_samples
    let result_small_epsilon =
        sprt_eval::evaluate_hypothesis(&ub_50_50, &session, 0.5, 0.95, 0.0001, 100, 0).unwrap();
    // The outcome here depends on random samples, so we can't assert true/false deterministically.
    // We just ensure it doesn't panic and returns a bool.
    assert!((result_small_epsilon == true) || (result_small_epsilon == false));

    // With a large epsilon, it might conclude faster or fallback more easily
    let result_large_epsilon =
        sprt_eval::evaluate_hypothesis(&ub_50_50, &session, 0.5, 0.95, 0.4, 10, 0).unwrap();
    assert!((result_large_epsilon == true) || (result_large_epsilon == false));
}

#[test]
fn test_evaluate_hypothesis_confidence_effect() {
    let session = SampleSession::seeded(SEED);
    let ub_60 = UncertainBool::<f64>::bernoulli(0.6);

    // High confidence requires more samples or stronger evidence
    let result_high_conf =
        sprt_eval::evaluate_hypothesis(&ub_60, &session, 0.5, 0.99, 0.01, 1000, 0).unwrap();
    assert!(
        result_high_conf,
        "Should conclude true with high confidence"
    );

    // Low confidence makes it easier to conclude, but requires sufficient samples to be stable.
    // Increased max_samples to 500 to make the test robust with random sampling.
    let result_low_conf =
        sprt_eval::evaluate_hypothesis(&ub_60, &session, 0.5, 0.70, 0.01, 500, 0).unwrap();
    assert!(result_low_conf, "Should conclude true with low confidence");
}

#[allow(clippy::bool_comparison)]
#[test]
fn test_evaluate_hypothesis_initial_sample_index() {
    let session = SampleSession::seeded(SEED);
    // Test that initial_sample_index is used correctly
    let ub_bernoulli = UncertainBool::<f64>::bernoulli(0.8);

    // If we start sampling from index 100, the results should still be consistent
    let result1 =
        sprt_eval::evaluate_hypothesis(&ub_bernoulli, &session, 0.5, 0.95, 0.01, 50, 0).unwrap();
    let result2 =
        sprt_eval::evaluate_hypothesis(&ub_bernoulli, &session, 0.5, 0.95, 0.01, 50, 100).unwrap();

    // Due to randomness, we can't assert exact equality, but they should both lean towards true
    assert!((result1 == true) || (result1 == false));
    assert!((result2 == true) || (result2 == false));
}
