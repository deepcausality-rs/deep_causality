/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::Float106;
use deep_causality_rand::{Distribution, Xoshiro256, rng};
use deep_causality_stats::{Bernoulli, BernoulliDistributionError};

#[test]
fn test_new() {
    // Valid cases
    let b = Bernoulli::new(0.5).unwrap();
    assert_eq!(b.p::<f64>(), 0.5);

    let b = Bernoulli::new(0.0).unwrap();
    assert_eq!(b.p::<f64>(), 0.0);

    let b = Bernoulli::new(1.0).unwrap();
    assert_eq!(b.p::<f64>(), 1.0);

    // Close to 1.0 but not 1.0
    let p_close_to_1 = 1.0 - 1e-12;
    let b = Bernoulli::new(p_close_to_1).unwrap();
    assert!((b.p::<f64>() - p_close_to_1).abs() < 1e-9);

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
    assert!((b.p::<f64>() - 0.5).abs() < f64::EPSILON);

    let b = Bernoulli::from_ratio(0, 1).unwrap();
    assert_eq!(b.p::<f64>(), 0.0);

    let b = Bernoulli::from_ratio(1, 1).unwrap();
    assert_eq!(b.p::<f64>(), 1.0);

    let b = Bernoulli::from_ratio(2, 3).unwrap();
    assert!((b.p::<f64>() - 2.0 / 3.0).abs() < f64::EPSILON);

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
    assert!((b.p::<f64>() - p).abs() < 1e-15);
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

// ---------------------------------------------------------------------------------------------
// The probability is stated in the caller's scalar, and quantised regardless
// ---------------------------------------------------------------------------------------------

#[test]
fn the_probability_is_stated_at_the_callers_scalar() {
    // The parameter is a probability, not an `f64`. Every supported scalar states it, and the
    // endpoints stay exact in each.
    let at_f32 = Bernoulli::new(0.25f32).unwrap();
    let at_f64 = Bernoulli::new(0.25f64).unwrap();
    assert_eq!(
        at_f32.p::<f64>(),
        at_f64.p::<f64>(),
        "0.25 is exact in both scalars, so both must quantise to the same probability"
    );

    assert!(
        Bernoulli::new(1.0f32)
            .unwrap()
            .sample(&mut Xoshiro256::from_seed(1))
    );
    assert!(
        !Bernoulli::new(0.0f32)
            .unwrap()
            .sample(&mut Xoshiro256::from_seed(1))
    );
}

#[test]
fn a_wider_scalar_buys_no_finer_probability() {
    // The fixed-width exception, measured. `Float106` can state a probability far finer than
    // `2^-64`, and the integer comparison cannot honour it: a probability and that probability
    // plus `2^-80` are the same distribution here.
    //
    // This is not a defect to fix by widening the store. It is the price of an integer comparison
    // that is exact at both endpoints, and the reason it is written down at the constructor.
    let p = Float106::from(0.5);
    let nudged = p + Float106::from(2f64.powi(-80));
    assert_ne!(p, nudged, "the two probabilities differ at Float106");

    let a = Bernoulli::new(p).unwrap();
    let b = Bernoulli::new(nudged).unwrap();
    assert_eq!(
        a, b,
        "two probabilities closer than 2^-64 must quantise to the same distribution"
    );
}
