/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{SampledValue, Uncertain, with_global_cache};

use rusty_fork::rusty_fork_test;

rusty_fork_test! {

#[test]
fn test_expected_value_zero_samples() {
    let uncertain = Uncertain::<f64>::point(42.0);
    assert_eq!(uncertain.expected_value(0).unwrap(), 0.0);
}

#[test]
fn test_expected_value_single_sample() {
    let uncertain = Uncertain::<f64>::point(42.0);
    assert_eq!(uncertain.expected_value(1).unwrap(), 42.0);
}

#[test]
fn test_expected_value_multiple_samples_constant() {
    let uncertain = Uncertain::<f64>::point(42.0);
    assert_eq!(uncertain.expected_value(100).unwrap(), 42.0);
}

#[test]
fn test_expected_value_multiple_samples_sequence() {
    let uncertain_seq = Uncertain::<f64>::point(999.0); // Changed from 0.0 to 999.0
    let id = uncertain_seq.id();

    with_global_cache(|cache| {
        cache.clear(); // Ensure a clean slate for this test
        for i in 0..5 { // Samples: 0, 1, 2, 3, 4
            cache.insert((id, i as u64), SampledValue::Float(i as f64));
        }
    });

    // Samples: 0, 1, 2, 3, 4. Sum = 10. Mean = 10 / 5 = 2.0
    assert_eq!(uncertain_seq.expected_value(5).unwrap(), 2.0);
}

#[test]
fn test_standard_deviation_zero_samples() {
    let uncertain = Uncertain::<f64>::point(42.0);
    assert_eq!(uncertain.standard_deviation(0).unwrap(), 0.0);
}

#[test]
fn test_standard_deviation_one_sample() {
    let uncertain = Uncertain::<f64>::point(42.0);
    assert_eq!(uncertain.standard_deviation(1).unwrap(), 0.0);
}

#[test]
fn test_standard_deviation_multiple_samples_constant() {
    let uncertain = Uncertain::<f64>::point(42.0);
    // All samples are the same, so variance and std dev are 0.
    assert_eq!(uncertain.standard_deviation(100).unwrap(), 0.0);
}

#[test]
fn test_standard_deviation_multiple_samples_sequence() {
    let uncertain_seq = Uncertain::<f64>::point(999.0); // Changed from 0.0 to 999.0
    let id = uncertain_seq.id();

    with_global_cache(|cache| {
        cache.clear(); // Ensure a clean slate for this test
        for i in 0..5 { // Samples: 0, 1, 2, 3, 4
            cache.insert((id, i as u64), SampledValue::Float(i as f64));
        }
    });

    // Samples: 0, 1, 2, 3, 4
    // Mean = 2.0
    // Squared deviations from mean: (0-2)^2=4, (1-2)^2=1, (2-2)^2=0, (3-2)^2=1, (4-2)^2=4
    // Sum of squared deviations = 10
    // Sample variance = 10 / (5-1) = 2.5
    // Sample standard deviation = sqrt(2.5)
    let std_dev = uncertain_seq.standard_deviation(5).unwrap();
    let expected_variance = 2.5f64;
    assert!((std_dev - expected_variance.sqrt()).abs() < 1e-9);
}
}
