/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{CausalTensor, CausalTensorError, CausalTensorStatsExt};
use std::f64::consts::PI;

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

// --- logsumexp ---

#[test]
fn logsumexp_matches_naive_for_small_inputs() {
    let t = CausalTensor::<f64>::new(vec![0.1, 0.2, 0.3, 0.4], vec![4, 1]).unwrap();
    let naive = t.as_slice().iter().map(|x| x.exp()).sum::<f64>().ln();
    assert!((t.logsumexp() - naive).abs() < EPS);
}

#[test]
fn logsumexp_equals_ln_n_for_zeros() {
    let t = CausalTensor::<f64>::new(vec![0.0, 0.0, 0.0], vec![3, 1]).unwrap();
    assert!((t.logsumexp() - 3.0_f64.ln()).abs() < EPS);
}

#[test]
fn logsumexp_does_not_overflow_for_large_inputs() {
    // exp(1000) overflows f64; the max-shift form must stay finite.
    let t = CausalTensor::<f64>::new(vec![1000.0, 1001.0], vec![2, 1]).unwrap();
    let lse = t.logsumexp();
    assert!(lse.is_finite());
    // log(e^1000 + e^1001) = 1001 + log(1 + e^-1)
    let expected = 1001.0 + (1.0 + (-1.0_f64).exp()).ln();
    assert!((lse - expected).abs() < 1e-9);
}

#[test]
fn logsumexp_single_element_is_that_element() {
    let t = CausalTensor::<f64>::new(vec![42.0], vec![1, 1]).unwrap();
    assert!((t.logsumexp() - 42.0).abs() < EPS);
}

#[test]
fn logsumexp_empty_is_negative_infinity() {
    let t = CausalTensor::<f64>::new(vec![], vec![0, 1]).unwrap();
    let lse = t.logsumexp();
    assert!(lse.is_infinite() && lse < 0.0);
}

#[test]
fn logsumexp_non_finite_max_returns_max() {
    let t = CausalTensor::<f64>::new(vec![1.0, f64::INFINITY, 2.0], vec![3, 1]).unwrap();
    assert!(t.logsumexp().is_infinite());
}

// --- gaussian_log_density ---

#[test]
fn gaussian_log_density_matches_closed_form() {
    // Standard normal evaluated at x = 0: −½·log(2π).
    let t = CausalTensor::<f64>::new(vec![0.0], vec![1, 1]).unwrap();
    let dens = t.gaussian_log_density(0.0, 1.0).unwrap();
    let expected = -0.5 * (2.0 * PI).ln();
    assert!((dens.as_slice()[0] - expected).abs() < EPS);
}

#[test]
fn gaussian_log_density_is_elementwise() {
    let t = CausalTensor::<f64>::new(vec![0.0, 1.0, -1.0], vec![3, 1]).unwrap();
    let dens = t.gaussian_log_density(0.0, 1.0).unwrap();
    assert_eq!(dens.shape(), &[3, 1]);
    for (x, lp) in t.as_slice().iter().zip(dens.as_slice().iter()) {
        let expected = -0.5 * ((2.0 * PI).ln() + x * x);
        assert!((lp - expected).abs() < EPS);
    }
}

#[test]
fn gaussian_log_density_floors_zero_variance() {
    let t = CausalTensor::<f64>::new(vec![0.5], vec![1, 1]).unwrap();
    let dens = t.gaussian_log_density(0.0, 0.0).unwrap();
    assert!(dens.as_slice()[0].is_finite());
}

#[test]
fn gaussian_log_density_floors_negative_variance() {
    let t = CausalTensor::<f64>::new(vec![0.5], vec![1, 1]).unwrap();
    let dens = t.gaussian_log_density(0.0, -3.0).unwrap();
    assert!(dens.as_slice()[0].is_finite());
}

#[test]
fn gaussian_log_density_empty_tensor_is_empty() {
    let t = CausalTensor::<f64>::new(vec![], vec![0, 1]).unwrap();
    let dens = t.gaussian_log_density(0.0, 1.0).unwrap();
    assert!(dens.is_empty());
}

// --- conditional_variance ---

#[test]
fn conditional_variance_empty_parents_is_marginal() {
    // Diagonal entry is the marginal variance.
    let cov = CausalTensor::<f64>::new(vec![2.0, 0.5, 0.5, 3.0], vec![2, 2]).unwrap();
    let cv = cov.conditional_variance(1, &[], 0.0).unwrap();
    assert!((cv - 3.0).abs() < EPS);
}

#[test]
fn conditional_variance_single_parent_closed_form() {
    // Σ = [[2,1],[1,1]]; Var(y|p) = σ_yy − σ_yp²/σ_pp = 2 − 1 = 1.
    let cov = CausalTensor::<f64>::new(vec![2.0, 1.0, 1.0, 1.0], vec![2, 2]).unwrap();
    let cv = cov.conditional_variance(0, &[1], 0.0).unwrap();
    assert!((cv - 1.0).abs() < EPS);
}

#[test]
fn conditional_variance_two_parent_identity_block() {
    // Σ_PP = I₂, Σ_yP = [a, b] → Var(y|P) = σ_yy − (a² + b²).
    // Variables: y=0, p1=1, p2=2. σ_yy=5, σ_y1=1, σ_y2=2, parents uncorrelated unit variance.
    let cov = CausalTensor::<f64>::new(
        vec![
            5.0, 1.0, 2.0, // y row
            1.0, 1.0, 0.0, // p1 row
            2.0, 0.0, 1.0, // p2 row
        ],
        vec![3, 3],
    )
    .unwrap();
    let cv = cov.conditional_variance(0, &[1, 2], 0.0).unwrap();
    // 5 − (1² + 2²) = 0
    assert!((cv - 0.0).abs() < 1e-10);
}

#[test]
fn conditional_variance_three_parent_identity_block() {
    // Σ_PP = I₃, Σ_yP = [a, b, c] → Var(y|P) = σ_yy − (a² + b² + c²).
    // Three parents force the Cholesky factorization to accumulate the
    // below-diagonal inner sum (the L[i,p]·L[j,p] term for p < j).
    // y=0, p1=1, p2=2, p3=3. σ_yy=14, σ_yP=[1,2,3], parents = unit identity block.
    let cov = CausalTensor::<f64>::new(
        vec![
            14.0, 1.0, 2.0, 3.0, // y row
            1.0, 1.0, 0.0, 0.0, // p1 row
            2.0, 0.0, 1.0, 0.0, // p2 row
            3.0, 0.0, 0.0, 1.0, // p3 row
        ],
        vec![4, 4],
    )
    .unwrap();
    let cv = cov.conditional_variance(0, &[1, 2, 3], 0.0).unwrap();
    // 14 − (1² + 2² + 3²) = 14 − 14 = 0
    assert!(cv.abs() < 1e-10);
}

#[test]
fn conditional_variance_three_parent_correlated_block() {
    // A non-diagonal 3-parent block keeps the Cholesky below-diagonal terms
    // non-trivial. Σ_PP = [[2,1,0],[1,2,1],[0,1,2]], Σ_yP = [1,1,1], σ_yy = 3.
    // Solve (Σ_PP) z = Σ_yP, then reduction = Σ_yP · z; cv = σ_yy − reduction.
    // For this block, z = [0.5, 0, 0.5] and reduction = 1.0 → cv = 2.0.
    let cov = CausalTensor::<f64>::new(
        vec![
            3.0, 1.0, 1.0, 1.0, // y
            1.0, 2.0, 1.0, 0.0, // p1
            1.0, 1.0, 2.0, 1.0, // p2
            1.0, 0.0, 1.0, 2.0, // p3
        ],
        vec![4, 4],
    )
    .unwrap();
    let cv = cov.conditional_variance(0, &[1, 2, 3], 0.0).unwrap();
    assert!(cv.is_finite());
    assert!((cv - 2.0).abs() < 1e-9, "expected 2.0, got {cv}");
}

#[test]
fn conditional_variance_singular_block_stabilized_by_ridge() {
    // Σ_PP = [[1,1],[1,1]] is singular; ridge keeps the solve finite.
    let cov = CausalTensor::<f64>::new(
        vec![
            2.0, 1.0, 1.0, //
            1.0, 1.0, 1.0, //
            1.0, 1.0, 1.0, //
        ],
        vec![3, 3],
    )
    .unwrap();
    let cv = cov.conditional_variance(0, &[1, 2], 1e-6).unwrap();
    assert!(cv.is_finite());
    // With ridge → 0, z ≈ [0.5, 0.5], reduction ≈ 1, cv ≈ 1.
    assert!((cv - 1.0).abs() < 1e-3);
}

#[test]
fn conditional_variance_rejects_non_square_matrix() {
    let cov = CausalTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();
    let err = cov
        .conditional_variance(0, &[1], 0.0)
        .expect_err("non-square covariance must be rejected");
    assert!(matches!(err, CausalTensorError::DimensionMismatch));
}

#[test]
fn conditional_variance_rejects_non_2d_matrix() {
    let cov = CausalTensor::<f64>::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let err = cov
        .conditional_variance(0, &[1], 0.0)
        .expect_err("1-D covariance must be rejected");
    assert!(matches!(err, CausalTensorError::DimensionMismatch));
}

#[test]
fn conditional_variance_rejects_out_of_range_target() {
    let cov = CausalTensor::<f64>::new(vec![2.0, 1.0, 1.0, 1.0], vec![2, 2]).unwrap();
    let err = cov
        .conditional_variance(5, &[1], 0.0)
        .expect_err("out-of-range target must be rejected");
    assert!(matches!(err, CausalTensorError::IndexOutOfBounds));
}

#[test]
fn conditional_variance_rejects_out_of_range_parent() {
    let cov = CausalTensor::<f64>::new(vec![2.0, 1.0, 1.0, 1.0], vec![2, 2]).unwrap();
    let err = cov
        .conditional_variance(0, &[7], 0.0)
        .expect_err("out-of-range parent must be rejected");
    assert!(matches!(err, CausalTensorError::IndexOutOfBounds));
}
