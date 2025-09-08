/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_uncertain::{SampledValue, Uncertain, with_global_cache};
use rusty_fork::rusty_fork_test;

rusty_fork_test! {
//
// Tests for Uncertain<f64>
//
#[test]
fn test_f64_sample_with_index_point() {
    let u = Uncertain::<f64>::point(42.0);
    let result = u.sample_with_index(0);
    assert_eq!(result.unwrap(), 42.0);
}

#[test]
fn test_from_sample() {
    let u = Uncertain::from_samples(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    let result = u.sample_with_index(0);
    assert!(result.is_ok());
}

#[test]
fn test_from_sample_empty() {
    let u = Uncertain::from_samples(&[]);
    let result = u.sample_with_index(0);
    assert!(result.is_ok());
}

#[test]
fn test_f64_sample_with_index_uses_cache() {
    // Use a non-deterministic distribution to verify caching.
    let u = Uncertain::uniform(0.0, 100.0);

    // First sample should compute the value and store it in the cache.
    let val1 = u.sample_with_index(123).unwrap();

    // Verify that the value is now in the cache.
    let key = (u.id(), 123);
    let cached_val = with_global_cache(|cache| cache.get(&key)).unwrap();
    match cached_val {
        SampledValue::Float(f) => assert_eq!(f, val1),
        _ => panic!("Cached value has the wrong type!"),
    }

    // A second sample at the same index should return the exact same value from the cache.
    let val2 = u.sample_with_index(123).unwrap();
    assert_eq!(val1, val2, "Value should be read from cache");
}

#[test]
fn test_f64_sample_with_random_index() {
    let u = Uncertain::<f64>::point(7.0);
    // We can't know the random index used by `sample`, but for a point distribution,
    // the result should always be the same.
    let result = u.sample();
    assert_eq!(result.unwrap(), 7.0);
}

#[test]
fn test_f64_take_samples() {
    let u = Uncertain::<f64>::point(88.0);
    let samples = u.take_samples(10).unwrap();
    assert_eq!(samples.len(), 10);
    assert!(samples.iter().all(|&s| s == 88.0));
}

#[test]
fn test_f64_take_zero_samples() {
    let u = Uncertain::<f64>::point(88.0);
    let samples = u.take_samples(0).unwrap();
    assert!(samples.is_empty());
}

#[test]
fn test_estimate_probability_exceeds_normal() {
    let u = Uncertain::normal(0.0, 1.0); // Standard normal distribution
    let threshold = 0.0;
    let num_samples = 10000;
    let prob = u
        .estimate_probability_exceeds(threshold, num_samples)
        .unwrap();
    // For a standard normal distribution, P(X > 0) should be close to 0.5
    assert!(
        (prob - 0.5).abs() < 0.05,
        "Expected probability near 0.5, got {}",
        prob
    );

    let threshold_high = 2.0;
    let prob_high = u
        .estimate_probability_exceeds(threshold_high, num_samples)
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
fn test_bool_sample_with_index_point() {
    let u_true = Uncertain::<bool>::point(true);
    assert!(u_true.sample_with_index(0).unwrap());

    let u_false = Uncertain::<bool>::point(false);
    assert!(!u_false.sample_with_index(1).unwrap());
}

#[test]
fn test_bool_sample_with_index_uses_cache() {
    // Use a non-deterministic distribution to verify caching.
    let u = Uncertain::bernoulli(0.5);

    // First sample.
    let val1 = u.sample_with_index(456).unwrap();

    // Verify cache.
    let key = (u.id(), 456);
    let cached_val = with_global_cache(|cache| cache.get(&key)).unwrap();
    match cached_val {
        SampledValue::Bool(b) => assert_eq!(b, val1),
        _ => panic!("Cached value has the wrong type!"),
    }

    // Second sample should be identical due to caching.
    let val2 = u.sample_with_index(456).unwrap();
    assert_eq!(val1, val2, "Value should be read from cache");
}

#[test]
fn test_bool_sample_with_random_index() {
    let u = Uncertain::<bool>::point(true);
    let result = u.sample();
    assert!(result.unwrap());
}

#[test]
fn test_bool_take_samples() {
    let u = Uncertain::<bool>::point(false);
    let samples = u.take_samples(20).unwrap();
    assert_eq!(samples.len(), 20);
    assert!(samples.iter().all(|&s| !s));
}

#[test]
fn test_bool_take_zero_samples() {
    let u = Uncertain::<bool>::point(true);
    let samples = u.take_samples(0).unwrap();
    assert!(samples.is_empty());
}

}
