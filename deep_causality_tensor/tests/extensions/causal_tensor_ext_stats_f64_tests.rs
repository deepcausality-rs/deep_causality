/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{CausalTensor, CausalTensorError, CausalTensorStatsExt};

const EPS: f64 = 1e-12;

// --- sample_mean ---

#[test]
fn sample_mean_returns_column_means() {
    // 3 observations x 2 variables.
    // col0 = [1,3,5] -> mean 3 ; col1 = [2,4,6] -> mean 4
    let t = CausalTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![3, 2]).unwrap();
    let means = t.sample_mean().expect("mean computes");
    assert_eq!(means.shape(), &[2]);
    assert!((means.as_slice()[0] - 3.0).abs() < EPS);
    assert!((means.as_slice()[1] - 4.0).abs() < EPS);
}

#[test]
fn sample_mean_single_observation_is_allowed() {
    // A single observation has a well-defined mean (it is the observation).
    let t = CausalTensor::<f64>::new(vec![7.0, 9.0], vec![1, 2]).unwrap();
    let means = t.sample_mean().expect("mean computes for one observation");
    assert_eq!(means.shape(), &[2]);
    assert!((means.as_slice()[0] - 7.0).abs() < EPS);
    assert!((means.as_slice()[1] - 9.0).abs() < EPS);
}

#[test]
fn sample_mean_rejects_non_2d_tensor() {
    let t = CausalTensor::<f64>::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let err = t.sample_mean().expect_err("1-D tensor must be rejected");
    assert!(matches!(err, CausalTensorError::DimensionMismatch));
}

#[test]
fn sample_mean_rejects_empty_tensor() {
    let t = CausalTensor::<f64>::new(vec![], vec![0, 2]).unwrap();
    let err = t.sample_mean().expect_err("empty tensor must be rejected");
    assert!(matches!(err, CausalTensorError::EmptyTensor));
}

// --- sample_covariance ---

#[test]
fn sample_covariance_matches_closed_form() {
    // 4 observations x 2 variables.
    // x1 = [1,2,3,4] mean 2.5 ; x2 = [2,1,4,3] mean 2.5
    // cov = [[5/3, 1], [1, 5/3]]
    let t =
        CausalTensor::<f64>::new(vec![1.0, 2.0, 2.0, 1.0, 3.0, 4.0, 4.0, 3.0], vec![4, 2]).unwrap();
    let cov = t.sample_covariance().expect("covariance computes");
    assert_eq!(cov.shape(), &[2, 2]);
    let five_thirds = 5.0 / 3.0;
    assert!((*cov.get(&[0, 0]).unwrap() - five_thirds).abs() < EPS);
    assert!((*cov.get(&[1, 1]).unwrap() - five_thirds).abs() < EPS);
    assert!((*cov.get(&[0, 1]).unwrap() - 1.0).abs() < EPS);
    // Covariance is symmetric.
    assert!((*cov.get(&[0, 1]).unwrap() - *cov.get(&[1, 0]).unwrap()).abs() < EPS);
}

#[test]
fn sample_covariance_diagonal_equals_per_column_variance() {
    // col0 = [1,3,5] -> variance ((−2)²+0+2²)/2 = 4
    // col1 = [2,4,6] -> variance 4
    let t = CausalTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![3, 2]).unwrap();
    let cov = t.sample_covariance().expect("covariance computes");
    assert!((*cov.get(&[0, 0]).unwrap() - 4.0).abs() < EPS);
    assert!((*cov.get(&[1, 1]).unwrap() - 4.0).abs() < EPS);
}

#[test]
fn sample_covariance_of_uniform_data_is_zero() {
    let t = CausalTensor::<f64>::new(vec![5.0, 5.0, 5.0, 5.0, 5.0, 5.0], vec![3, 2]).unwrap();
    let cov = t.sample_covariance().expect("covariance computes");
    for v in cov.as_slice() {
        assert!(v.abs() < 1e-10, "uniform data must have zero covariance");
    }
}

#[test]
fn sample_covariance_rejects_single_observation() {
    let t = CausalTensor::<f64>::new(vec![1.0, 2.0], vec![1, 2]).unwrap();
    let err = t
        .sample_covariance()
        .expect_err("single observation must be rejected");
    assert!(matches!(err, CausalTensorError::InvalidParameter(_)));
}

#[test]
fn sample_covariance_rejects_non_2d_tensor() {
    let t = CausalTensor::<f64>::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let err = t
        .sample_covariance()
        .expect_err("1-D tensor must be rejected");
    assert!(matches!(err, CausalTensorError::DimensionMismatch));
}

#[test]
fn sample_covariance_rejects_zero_column_tensor() {
    let t = CausalTensor::<f64>::new(vec![], vec![3, 0]).unwrap();
    let err = t
        .sample_covariance()
        .expect_err("zero-column tensor must be rejected");
    assert!(matches!(err, CausalTensorError::EmptyTensor));
}
