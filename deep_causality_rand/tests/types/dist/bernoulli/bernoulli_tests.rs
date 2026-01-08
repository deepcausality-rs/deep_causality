/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_rand::{Bernoulli, BernoulliDistributionError, Distribution, rng};

#[test]
fn test_new() {
    // Valid cases
    let b = Bernoulli::new(0.5).unwrap();
    assert_eq!(b.p(), 0.5);

    let b = Bernoulli::new(0.0).unwrap();
    assert_eq!(b.p(), 0.0);

    let b = Bernoulli::new(1.0).unwrap();
    assert_eq!(b.p(), 1.0);

    // Close to 1.0 but not 1.0
    let p_close_to_1 = 1.0 - 1e-12;
    let b = Bernoulli::new(p_close_to_1).unwrap();
    assert!((b.p() - p_close_to_1).abs() < 1e-9);

    // Invalid cases
    assert_eq!(
        Bernoulli::new(-0.1).unwrap_err(),
        BernoulliDistributionError::InvalidProbability
    );
    assert_eq!(
        Bernoulli::new(1.1).unwrap_err(),
        BernoulliDistributionError::InvalidProbability
    );
    assert!(Bernoulli::new(f64::NAN).is_err());
}

#[test]
fn test_from_ratio() {
    // Valid cases
    let b = Bernoulli::from_ratio(1, 2).unwrap();
    assert!((b.p() - 0.5).abs() < f64::EPSILON);

    let b = Bernoulli::from_ratio(0, 1).unwrap();
    assert_eq!(b.p(), 0.0);

    let b = Bernoulli::from_ratio(1, 1).unwrap();
    assert_eq!(b.p(), 1.0);

    let b = Bernoulli::from_ratio(2, 3).unwrap();
    assert!((b.p() - 2.0 / 3.0).abs() < f64::EPSILON);

    // Invalid cases
    assert_eq!(
        Bernoulli::from_ratio(2, 1).unwrap_err(),
        BernoulliDistributionError::InvalidProbability
    );
    assert_eq!(
        Bernoulli::from_ratio(1, 0).unwrap_err(),
        BernoulliDistributionError::InvalidProbability
    );
}

#[test]
fn test_p_precision() {
    let p = 0.123456789;
    let b = Bernoulli::new(p).unwrap();
    // The precision of f64 is about 15-17 decimal digits.
    // The conversion to u64 and back might lose some precision.
    assert!((b.p() - p).abs() < 1e-15);
}

#[test]
fn test_sample_deterministic() {
    let mut rng = rng();

    // p = 1.0 should always be true
    let b_true = Bernoulli::new(1.0).unwrap();
    assert!(b_true.sample(&mut rng));
    assert!(b_true.sample(&mut rng));

    // p = 0.0 should always be false
    let b_false = Bernoulli::new(0.0).unwrap();
    assert!(!b_false.sample(&mut rng));
    assert!(!b_false.sample(&mut rng));
}

#[test]
fn test_clone_copy_debug_partial_eq() {
    let b1 = Bernoulli::new(0.25).unwrap();
    let b2 = b1; // Test Copy
    let b3 = b1; // Test Clone
    assert_eq!(b1, b2);
    assert_eq!(b1, b3);

    let b4 = Bernoulli::new(0.75).unwrap();
    assert_ne!(b1, b4);

    const SCALE: f64 = 2.0 * (1u64 << 63) as f64;
    let p_int = (0.25 * SCALE) as u64;
    assert_eq!(
        format!("{:?}", b1),
        format!("Bernoulli {{ p_int: {} }}", p_int)
    );
}
