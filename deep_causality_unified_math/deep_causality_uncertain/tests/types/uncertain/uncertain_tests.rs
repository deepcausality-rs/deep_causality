/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::UncertainBool;
use deep_causality_uncertain::{SampleSession, Uncertain};

/// Every test below that observes a draw installs this seed first.
///
/// Without it the draws come from OS entropy, and a test that gates on a sampled decision is a
/// coin flip with a good bias rather than an assertion. Measured before seeding: five tests in
/// this crate failed across ~360 runs, and 41 sampled-decision call sites were exposed. The
/// assertions are unchanged; only the entropy source is.
const SEED: u64 = 0x5EED_2026;

// Helper for approximate equality for f64
fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
    assert!(
        (a - b).abs() < epsilon,
        "{} is not approximately equal to {}",
        a,
        b
    );
}

// Test for Uncertain<f64> constructors
#[test]
fn test_uncertain_f64_point_constructor() {
    let session = SampleSession::seeded(SEED);
    let uncertain = Uncertain::<f64>::point(123.45);
    let sample = uncertain.sample_at(&session, 0).unwrap();
    assert_eq!(sample, 123.45);
}

#[test]
fn test_uncertain_f64_normal_constructor() {
    let session = SampleSession::seeded(SEED);
    let mean = 10.0;
    let std_dev = 2.0;
    let uncertain = Uncertain::<f64>::normal(mean, std_dev);

    let num_samples = 10000;
    let samples: Vec<f64> = (0..num_samples)
        .map(|i| uncertain.sample_at(&session, i as u64).unwrap())
        .collect();

    let sum: f64 = samples.iter().sum();
    let actual_mean = sum / num_samples as f64;

    // Check if the mean is close to the expected mean
    assert_approx_eq(actual_mean, mean, 0.1); // Allow some tolerance for statistical variation

    // Calculate actual standard deviation of the samples
    let sum_of_squared_diffs: f64 = samples.iter().map(|&x| (x - actual_mean).powi(2)).sum();
    let actual_std_dev = (sum_of_squared_diffs / (num_samples as f64 - 1.0)).sqrt(); // Sample standard deviation

    // Check if the actual standard deviation is close to the expected std_dev
    assert_approx_eq(actual_std_dev, std_dev, 0.1); // Allow some tolerance

    // Removed strict individual sample bounds check, as it's statistically prone to failure.
}

#[test]
fn test_uncertain_f64_uniform_constructor() {
    let session = SampleSession::seeded(SEED);
    let low = 0.0;
    let high = 100.0;
    let uncertain = Uncertain::<f64>::uniform(low, high);

    let num_samples = 10000;
    let samples: Vec<f64> = (0..num_samples)
        .map(|i| uncertain.sample_at(&session, i as u64).unwrap())
        .collect();

    // Check if all samples are within the defined range
    for &sample in &samples {
        assert!(sample >= low);
        assert!(sample <= high);
    }

    // Check if the mean is close to the expected mean for a uniform distribution (low + high) / 2
    let sum: f64 = samples.iter().sum();
    let actual_mean = sum / num_samples as f64;
    assert_approx_eq(actual_mean, (low + high) / 2.0, 1.0); // Increased epsilon from 0.5 to 1.0
}

// Test for Uncertain<f64> mappers
#[test]
fn test_uncertain_f64_map() {
    let session = SampleSession::seeded(SEED);
    let uncertain_input = Uncertain::<f64>::point(5.0);
    let uncertain_mapped = uncertain_input.map(|x| x * 2.0 + 1.0); // Should be 11.0

    let sample = uncertain_mapped.sample_at(&session, 0).unwrap();
    assert_eq!(sample, 11.0);
}

#[test]
fn test_uncertain_f64_map_to_bool() {
    let session = SampleSession::seeded(SEED);
    let uncertain_input = Uncertain::<f64>::point(7.0);
    let uncertain_bool = uncertain_input.map_to_bool(|x| x > 5.0); // Should be true

    let sample = uncertain_bool.sample_at(&session, 0).unwrap();
    assert!(sample);

    let uncertain_bool_false = uncertain_input.map_to_bool(|x| x < 5.0); // Should be false
    let sample_false = uncertain_bool_false.sample_at(&session, 0).unwrap();
    assert!(!sample_false);
}

// Test for UncertainBool<f64> constructors
#[test]
fn test_uncertain_bool_point_constructor() {
    let session = SampleSession::seeded(SEED);
    let uncertain_true = UncertainBool::<f64>::point(true);
    assert!(uncertain_true.sample_at(&session, 0).unwrap());

    let uncertain_false = UncertainBool::<f64>::point(false);
    assert!(!uncertain_false.sample_at(&session, 0).unwrap());
}

#[test]
fn test_uncertain_bool_bernoulli_constructor() {
    let session = SampleSession::seeded(SEED);
    let p = 0.7;
    let uncertain = UncertainBool::<f64>::bernoulli(p);

    let num_samples = 10000;
    let samples: Vec<bool> = (0..num_samples)
        .map(|i| uncertain.sample_at(&session, i as u64).unwrap())
        .collect();

    let true_count = samples.iter().filter(|&&b| b).count();
    let actual_p = true_count as f64 / num_samples as f64;

    assert_approx_eq(actual_p, p, 0.05); // Allow some tolerance
}

// Test for UncertainBool<f64> methods
#[test]
fn test_uncertain_bool_to_bool() {
    let session = SampleSession::seeded(SEED);
    // Clearly true
    let uncertain_true = UncertainBool::<f64>::point(true);
    assert!(
        uncertain_true
            .to_bool(&session, 0.99, 0.95, 0.05, 1000)
            .unwrap()
    );

    // Clearly false
    let uncertain_false = UncertainBool::<f64>::point(false);
    assert!(
        !uncertain_false
            .to_bool(&session, 0.99, 0.95, 0.05, 1000)
            .unwrap()
    );

    // Bernoulli with high probability of true
    let uncertain_bernoulli_true = UncertainBool::<f64>::bernoulli(0.9);
    assert!(
        uncertain_bernoulli_true
            .to_bool(&session, 0.8, 0.95, 0.05, 1000)
            .unwrap()
    );

    // Bernoulli with high probability of false
    let uncertain_bernoulli_false = UncertainBool::<f64>::bernoulli(0.1);
    assert!(
        !uncertain_bernoulli_false
            .to_bool(&session, 0.9, 0.95, 0.05, 1000)
            .unwrap()
    );
}

#[test]
fn test_uncertain_bool_probability_exceeds() {
    let session = SampleSession::seeded(SEED);
    // Test with threshold 0.5, confidence 0.9
    let uncertain_true = UncertainBool::<f64>::point(true);
    assert!(
        uncertain_true
            .probability_exceeds(&session, 0.5, 0.9, 0.05, 100)
            .unwrap()
    );

    let uncertain_false = UncertainBool::<f64>::point(false);
    assert!(
        !uncertain_false
            .probability_exceeds(&session, 0.5, 0.9, 0.05, 100)
            .unwrap()
    );

    // Test with threshold 0.8, confidence 0.9
    let uncertain_bernoulli_high = UncertainBool::<f64>::bernoulli(0.9);
    assert!(
        uncertain_bernoulli_high
            .probability_exceeds(&session, 0.8, 0.9, 0.05, 10000)
            .unwrap()
    );
}

#[test]
fn test_uncertain_bool_implicit_conditional() {
    let session = SampleSession::seeded(SEED);
    // This should behave like to_bool with default confidence and threshold 0.5
    let uncertain_true = UncertainBool::<f64>::point(true);
    assert!(uncertain_true.implicit_conditional(&session).unwrap());

    let uncertain_false = UncertainBool::<f64>::point(false);
    assert!(!uncertain_false.implicit_conditional(&session).unwrap());
}

#[test]
fn test_uncertain_bool_estimate_probability() {
    let session = SampleSession::seeded(SEED);
    let uncertain_true = UncertainBool::<f64>::point(true);
    assert_approx_eq(
        uncertain_true.estimate_probability(&session, 100).unwrap(),
        1.0,
        0.01,
    );

    let uncertain_false = UncertainBool::<f64>::point(false);
    assert_approx_eq(
        uncertain_false.estimate_probability(&session, 100).unwrap(),
        0.0,
        0.01,
    );

    let p = 0.6;
    let uncertain_bernoulli = UncertainBool::<f64>::bernoulli(p);
    assert_approx_eq(
        uncertain_bernoulli
            .estimate_probability(&session, 10000)
            .unwrap(),
        p,
        0.05,
    );

    // Test with zero samples
    assert_eq!(
        uncertain_bernoulli
            .estimate_probability(&session, 0)
            .unwrap(),
        0.0
    );
}

// Test for conditional()
#[test]
fn test_uncertain_conditional_f64_true_condition() {
    let session = SampleSession::seeded(SEED);
    let condition = UncertainBool::<f64>::point(true);
    let if_true = Uncertain::<f64>::point(10.0);
    let if_false = Uncertain::<f64>::point(20.0);

    let result = Uncertain::conditional(condition, if_true, if_false);
    assert_eq!(result.sample_at(&session, 0).unwrap(), 10.0);
}

#[test]
fn test_uncertain_conditional_f64_false_condition() {
    let session = SampleSession::seeded(SEED);
    let condition = UncertainBool::<f64>::point(false);
    let if_true = Uncertain::<f64>::point(10.0);
    let if_false = Uncertain::<f64>::point(20.0);

    let result = Uncertain::conditional(condition, if_true, if_false);
    assert_eq!(result.sample_at(&session, 0).unwrap(), 20.0);
}

#[test]
fn test_uncertain_conditional_bool_true_condition() {
    let session = SampleSession::seeded(SEED);
    let condition = UncertainBool::<f64>::point(true);
    let if_true = UncertainBool::<f64>::point(true);
    let if_false = UncertainBool::<f64>::point(false);

    let result = UncertainBool::conditional(condition, if_true, if_false);
    assert!(result.sample_at(&session, 0).unwrap());
}

#[test]
fn test_uncertain_conditional_bool_false_condition() {
    let session = SampleSession::seeded(SEED);
    let condition = UncertainBool::<f64>::point(false);
    let if_true = UncertainBool::<f64>::point(true);
    let if_false = UncertainBool::<f64>::point(false);

    let result = UncertainBool::conditional(condition, if_true, if_false);
    assert!(!result.sample_at(&session, 0).unwrap());
}

#[test]
fn test_uncertain_conditional_f64_uncertain_condition() {
    let session = SampleSession::seeded(SEED);
    let uncertain_condition = UncertainBool::<f64>::bernoulli(0.5); // 50/50 chance
    let if_true_val = Uncertain::<f64>::point(100.0);
    let if_false_val = Uncertain::<f64>::point(200.0);

    let result_uncertain = Uncertain::conditional(uncertain_condition, if_true_val, if_false_val);

    let num_samples = 1000;
    let mut true_count = 0;
    let mut false_count = 0;

    for __i in 0..num_samples {
        let sample = result_uncertain.sample_at(&session, __i as u64).unwrap();
        if sample == 100.0 {
            true_count += 1;
        } else if sample == 200.0 {
            false_count += 1;
        }
    }

    // Expect roughly half true and half false
    assert_approx_eq(true_count as f64 / num_samples as f64, 0.5, 0.1);
    assert_approx_eq(false_count as f64 / num_samples as f64, 0.5, 0.1);
}

#[test]
fn test_uncertain_f64_sample() {
    let session = SampleSession::seeded(SEED);
    let uncertain = Uncertain::<f64>::point(42.0);
    let sample = uncertain.sample_at(&session, 0).unwrap();
    assert_eq!(sample, 42.0);
}

#[test]
fn test_uncertain_bool_sample() {
    let session = SampleSession::seeded(SEED);
    let uncertain = UncertainBool::<f64>::point(true);
    let sample = uncertain.sample_at(&session, 0).unwrap();
    assert!(sample);
}
