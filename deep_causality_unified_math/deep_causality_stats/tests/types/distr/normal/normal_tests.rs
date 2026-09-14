/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_rand::{Distribution, Xoshiro256};
use deep_causality_stats::Normal;
use deep_causality_stats::NormalDistributionError;
use deep_causality_stats::utils_tests::sampling::SUITE_SEED;

#[test]
fn test_f64_new() {
    let n = Normal::new(10.0, 2.0).unwrap();
    assert_eq!(n.mean(), 10.0);
    assert_eq!(n.std_dev(), 2.0);

    let res = Normal::new(10.0, f64::INFINITY);
    assert_eq!(res.unwrap_err(), NormalDistributionError::BadVariance);
}

#[test]
fn test_f64_from_mean_cv() {
    let n = Normal::from_mean_cv(10.0, 0.2).unwrap();
    assert_eq!(n.mean(), 10.0);
    assert_eq!(n.std_dev(), 2.0);

    let res = Normal::from_mean_cv(10.0, f64::INFINITY);
    assert_eq!(res.unwrap_err(), NormalDistributionError::BadVariance);

    let res = Normal::from_mean_cv(10.0, -0.1);
    assert_eq!(res.unwrap_err(), NormalDistributionError::BadVariance);
}

#[test]
fn a_zero_coefficient_of_variation_is_a_degenerate_normal() {
    // `new` admits a zero standard deviation — it rejects only a non-finite one — so
    // `from_mean_cv` must admit a zero coefficient of variation. The two constructors describe the
    // same family and cannot disagree at its boundary, and the existing cases here step over that
    // boundary: 0.2 on one side, -0.1 on the other, nothing at zero.
    let n = Normal::from_mean_cv(10.0, 0.0)
        .expect("a zero coefficient of variation is a degenerate normal, not a bad variance");
    assert_eq!(n.mean(), 10.0);
    assert_eq!(n.std_dev(), 0.0);

    // With no width, every draw is the mean.
    let mut g = Xoshiro256::from_seed(SUITE_SEED);
    for _ in 0..100 {
        let x: f64 = n.sample(&mut g);
        assert_eq!(
            x, 10.0,
            "a zero-width normal drew something other than its mean"
        );
    }
}

#[test]
fn test_f64_sample_from_zscore() {
    let n = Normal::new(10.0, 2.0).unwrap();
    assert_eq!(n.sample_from_zscore(0.0), 10.0);
    assert_eq!(n.sample_from_zscore(1.0), 12.0);
    assert_eq!(n.sample_from_zscore(-1.0), 8.0);
    assert_eq!(n.sample_from_zscore(2.5), 15.0);
}

#[test]
fn test_f64_getters() {
    let n = Normal::new(10.0, 2.0).unwrap();
    assert_eq!(n.mean(), 10.0);
    assert_eq!(n.std_dev(), 2.0);
}

#[test]
fn test_f64_derived_traits() {
    let n1 = Normal::new(10.0, 2.0).unwrap();
    let n2 = n1; // Copy
    let n3 = n1; // Clone
    assert_eq!(n1, n2);
    assert_eq!(n1, n3);

    let n4 = Normal::new(11.0, 2.0).unwrap();
    assert_ne!(n1, n4);

    assert_eq!(format!("{:?}", n1), "Normal { mean: 10.0, std_dev: 2.0 }");
}

// Now for f32
#[test]
fn test_f32_new() {
    let n = Normal::new(10.0f32, 2.0f32).unwrap();
    assert_eq!(n.mean(), 10.0f32);
    assert_eq!(n.std_dev(), 2.0f32);

    let res = Normal::new(10.0f32, f32::INFINITY);
    assert_eq!(res.unwrap_err(), NormalDistributionError::BadVariance);
}

#[test]
fn test_f32_from_mean_cv() {
    let n = Normal::from_mean_cv(10.0f32, 0.2f32).unwrap();
    assert_eq!(n.mean(), 10.0f32);
    assert_eq!(n.std_dev(), 2.0f32);

    let res = Normal::from_mean_cv(10.0f32, f32::INFINITY);
    assert_eq!(res.unwrap_err(), NormalDistributionError::BadVariance);

    let res = Normal::from_mean_cv(10.0f32, -0.1f32);
    assert_eq!(res.unwrap_err(), NormalDistributionError::BadVariance);
}

#[test]
fn test_f32_sample_from_zscore() {
    let n = Normal::new(10.0f32, 2.0f32).unwrap();
    assert_eq!(n.sample_from_zscore(0.0f32), 10.0f32);
    assert_eq!(n.sample_from_zscore(1.0f32), 12.0f32);
    assert_eq!(n.sample_from_zscore(-1.0f32), 8.0f32);
    assert_eq!(n.sample_from_zscore(2.5f32), 15.0f32);
}
