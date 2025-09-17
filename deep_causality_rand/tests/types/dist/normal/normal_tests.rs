/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_rand::Normal;
use deep_causality_rand::NormalDistributionError;

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
