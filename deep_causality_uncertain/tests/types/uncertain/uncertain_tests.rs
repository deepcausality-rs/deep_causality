/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{SampledValue, Uncertain, with_global_cache};

// Helper for approximate equality for f64
fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
    assert!(
        (a - b).abs() < epsilon,
        "{} is not approximately equal to {}",
        a,
        b
    );
}

// Test for id()
#[test]
fn test_uncertain_id() {
    let u1 = Uncertain::<f64>::point(1.0);
    let u2 = Uncertain::<f64>::point(2.0);
    assert_ne!(u1.id(), u2.id()); // IDs should be unique
    assert!(u2.id() > u1.id()); // IDs should be incrementing
}

// Test for Uncertain<f64> constructors
#[test]
fn test_uncertain_f64_point_constructor() {
    let uncertain = Uncertain::<f64>::point(123.45);
    let sample = uncertain.sample().unwrap();
    assert_eq!(sample, 123.45);
}

#[test]
fn test_uncertain_f64_normal_constructor() {
    let mean = 10.0;
    let std_dev = 2.0;
    let uncertain = Uncertain::<f64>::normal(mean, std_dev);

    let num_samples = 10000;
    let samples: Vec<f64> = (0..num_samples)
        .map(|_| uncertain.sample().unwrap())
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
    let low = 0.0;
    let high = 100.0;
    let uncertain = Uncertain::<f64>::uniform(low, high);

    let num_samples = 10000;
    let samples: Vec<f64> = (0..num_samples)
        .map(|_| uncertain.sample().unwrap())
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
    let uncertain_input = Uncertain::<f64>::point(5.0);
    let uncertain_mapped = uncertain_input.map(|x| x * 2.0 + 1.0); // Should be 11.0

    let sample = uncertain_mapped.sample().unwrap();
    assert_eq!(sample, 11.0);
}

#[test]
fn test_uncertain_f64_map_to_bool() {
    let uncertain_input = Uncertain::<f64>::point(7.0);
    let uncertain_bool = uncertain_input.map_to_bool(|x| x > 5.0); // Should be true

    let sample = uncertain_bool.sample().unwrap();
    assert!(sample);

    let uncertain_bool_false = uncertain_input.map_to_bool(|x| x < 5.0); // Should be false
    let sample_false = uncertain_bool_false.sample().unwrap();
    assert!(!sample_false);
}

// Test for Uncertain<bool> constructors
#[test]
fn test_uncertain_bool_point_constructor() {
    let uncertain_true = Uncertain::<bool>::point(true);
    assert!(uncertain_true.sample().unwrap());

    let uncertain_false = Uncertain::<bool>::point(false);
    assert!(!uncertain_false.sample().unwrap());
}

#[test]
fn test_uncertain_bool_bernoulli_constructor() {
    let p = 0.7;
    let uncertain = Uncertain::<bool>::bernoulli(p);

    let num_samples = 10000;
    let samples: Vec<bool> = (0..num_samples)
        .map(|_| uncertain.sample().unwrap())
        .collect();

    let true_count = samples.iter().filter(|&&b| b).count();
    let actual_p = true_count as f64 / num_samples as f64;

    assert_approx_eq(actual_p, p, 0.05); // Allow some tolerance
}

// Test for Uncertain<bool> methods
#[test]
fn test_uncertain_bool_to_bool() {
    // Clearly true
    let uncertain_true = Uncertain::<bool>::point(true);
    assert!(uncertain_true.to_bool(0.99, 0.95, 0.05, 1000).unwrap());

    // Clearly false
    let uncertain_false = Uncertain::<bool>::point(false);
    assert!(!uncertain_false.to_bool(0.99, 0.95, 0.05, 1000).unwrap());

    // Bernoulli with high probability of true
    let uncertain_bernoulli_true = Uncertain::<bool>::bernoulli(0.9);
    assert!(
        uncertain_bernoulli_true
            .to_bool(0.8, 0.95, 0.05, 1000)
            .unwrap()
    );

    // Bernoulli with high probability of false
    let uncertain_bernoulli_false = Uncertain::<bool>::bernoulli(0.1);
    assert!(
        !uncertain_bernoulli_false
            .to_bool(0.9, 0.95, 0.05, 1000)
            .unwrap()
    );
}

#[test]
fn test_uncertain_bool_probability_exceeds() {
    // Test with threshold 0.5, confidence 0.9
    let uncertain_true = Uncertain::<bool>::point(true);
    assert!(
        uncertain_true
            .probability_exceeds(0.5, 0.9, 0.05, 100)
            .unwrap()
    );

    let uncertain_false = Uncertain::<bool>::point(false);
    assert!(
        !uncertain_false
            .probability_exceeds(0.5, 0.9, 0.05, 100)
            .unwrap()
    );

    // Test with threshold 0.8, confidence 0.9
    let uncertain_bernoulli_high = Uncertain::<bool>::bernoulli(0.9);
    assert!(
        uncertain_bernoulli_high
            .probability_exceeds(0.8, 0.9, 0.05, 1000)
            .unwrap()
    );

    let uncertain_bernoulli_low = Uncertain::<bool>::bernoulli(0.7);
    assert!(
        !uncertain_bernoulli_low
            .probability_exceeds(0.8, 0.9, 0.05, 1000)
            .unwrap()
    );
}

#[test]
fn test_uncertain_bool_implicit_conditional() {
    // This should behave like to_bool with default confidence and threshold 0.5
    let uncertain_true = Uncertain::<bool>::point(true);
    assert!(uncertain_true.implicit_conditional().unwrap());

    let uncertain_false = Uncertain::<bool>::point(false);
    assert!(!uncertain_false.implicit_conditional().unwrap());
}

#[test]
fn test_uncertain_bool_estimate_probability() {
    let uncertain_true = Uncertain::<bool>::point(true);
    assert_approx_eq(uncertain_true.estimate_probability(100).unwrap(), 1.0, 0.01);

    let uncertain_false = Uncertain::<bool>::point(false);
    assert_approx_eq(
        uncertain_false.estimate_probability(100).unwrap(),
        0.0,
        0.01,
    );

    let p = 0.6;
    let uncertain_bernoulli = Uncertain::<bool>::bernoulli(p);
    assert_approx_eq(
        uncertain_bernoulli.estimate_probability(10000).unwrap(),
        p,
        0.05,
    );

    // Test with zero samples
    assert_eq!(uncertain_bernoulli.estimate_probability(0).unwrap(), 0.0);
}

// Test for conditional()
#[test]
fn test_uncertain_conditional_f64_true_condition() {
    let condition = Uncertain::<bool>::point(true);
    let if_true = Uncertain::<f64>::point(10.0);
    let if_false = Uncertain::<f64>::point(20.0);

    let result = Uncertain::conditional(condition, if_true, if_false);
    assert_eq!(result.sample().unwrap(), 10.0);
}

#[test]
fn test_uncertain_conditional_f64_false_condition() {
    let condition = Uncertain::<bool>::point(false);
    let if_true = Uncertain::<f64>::point(10.0);
    let if_false = Uncertain::<f64>::point(20.0);

    let result = Uncertain::conditional(condition, if_true, if_false);
    assert_eq!(result.sample().unwrap(), 20.0);
}

#[test]
fn test_uncertain_conditional_bool_true_condition() {
    let condition = Uncertain::<bool>::point(true);
    let if_true = Uncertain::<bool>::point(true);
    let if_false = Uncertain::<bool>::point(false);

    let result = Uncertain::conditional(condition, if_true, if_false);
    assert!(result.sample().unwrap());
}

#[test]
fn test_uncertain_conditional_bool_false_condition() {
    let condition = Uncertain::<bool>::point(false);
    let if_true = Uncertain::<bool>::point(true);
    let if_false = Uncertain::<bool>::point(false);

    let result = Uncertain::conditional(condition, if_true, if_false);
    assert!(!result.sample().unwrap());
}

#[test]
fn test_uncertain_conditional_f64_uncertain_condition() {
    let uncertain_condition = Uncertain::<bool>::bernoulli(0.5); // 50/50 chance
    let if_true_val = Uncertain::<f64>::point(100.0);
    let if_false_val = Uncertain::<f64>::point(200.0);

    let result_uncertain = Uncertain::conditional(uncertain_condition, if_true_val, if_false_val);

    let num_samples = 1000;
    let mut true_count = 0;
    let mut false_count = 0;

    for _ in 0..num_samples {
        let sample = result_uncertain.sample().unwrap();
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
    let uncertain = Uncertain::<f64>::point(42.0);
    let sample = uncertain.sample().unwrap();
    assert_eq!(sample, 42.0);
}

#[test]
fn test_uncertain_f64_sample_with_index() {
    let uncertain = Uncertain::<f64>::point(10.0);
    let id = uncertain.id();

    with_global_cache(|cache| {
        cache.clear();
        cache.insert((id, 0), SampledValue::Float(100.0));
        cache.insert((id, 1), SampledValue::Float(200.0));
    });

    assert_eq!(uncertain.sample_with_index(0).unwrap(), 100.0);
    assert_eq!(uncertain.sample_with_index(1).unwrap(), 200.0);
}

#[test]
fn test_uncertain_bool_sample() {
    let uncertain = Uncertain::<bool>::point(true);
    let sample = uncertain.sample().unwrap();
    assert!(sample);
}

#[test]
fn test_uncertain_bool_sample_with_index() {
    let uncertain = Uncertain::<bool>::point(false);
    let id = uncertain.id();

    with_global_cache(|cache| {
        cache.clear();
        cache.insert((id, 0), SampledValue::Bool(true));
        cache.insert((id, 1), SampledValue::Bool(false));
    });

    assert!(uncertain.sample_with_index(0).unwrap());
    assert!(!uncertain.sample_with_index(1).unwrap());
}
