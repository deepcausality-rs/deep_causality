/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::{BicScorer, BossConfig, BrcdErrorEnum, family_bic};
use deep_causality_tensor::{CausalTensor, CausalTensorStatsExt};

const RIDGE: f64 = 1e-6;
const LAMBDA: f64 = 2.0;

/// Row-major `n × k` data tensor from `k` equal-length columns.
fn data_from_columns(cols: &[&[f64]]) -> CausalTensor<f64> {
    let n = cols[0].len();
    let k = cols.len();
    let mut flat = Vec::with_capacity(n * k);
    for i in 0..n {
        for c in cols {
            flat.push(c[i]);
        }
    }
    CausalTensor::from_slice(&flat, &[n, k])
}

fn approx(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "expected {b}, got {a}");
}

// X and Y = 2X + small residual; a strong linear parent.
fn xy_data() -> (CausalTensor<f64>, usize) {
    let x = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let y = [0.1, 1.9, 4.05, 5.95, 8.1, 9.9, 12.05, 13.95];
    (data_from_columns(&[&x, &y]), x.len())
}

#[test]
fn no_parent_score_matches_the_closed_form() {
    let (data, n) = xy_data();
    let cov = data.sample_covariance().unwrap();

    // σ² for the no-parent case is the marginal variance cov[1,1].
    let var = cov.conditional_variance(1, &[], RIDGE).unwrap();
    let expected = -0.5 * (n as f64) * var.ln() - 0.5 * (n as f64).ln() * 1.0 * LAMBDA;

    let got = family_bic(&cov, n, 1, &[], RIDGE, LAMBDA).unwrap();
    approx(got, expected);
}

#[test]
fn parented_score_matches_the_closed_form() {
    let (data, n) = xy_data();
    let cov = data.sample_covariance().unwrap();

    let var = cov.conditional_variance(1, &[0], RIDGE).unwrap();
    // Penalty multiplies (|PA| + 1) = 2.
    let expected = -0.5 * (n as f64) * var.ln() - 0.5 * (n as f64).ln() * 2.0 * LAMBDA;

    let got = family_bic(&cov, n, 1, &[0], RIDGE, LAMBDA).unwrap();
    approx(got, expected);
}

#[test]
fn score_is_higher_is_better_so_a_real_parent_wins() {
    let (data, n) = xy_data();
    let cov = data.sample_covariance().unwrap();

    // A parent that sharply reduces the conditional variance must score strictly
    // higher than the empty set — otherwise grow/shrink would never add it.
    let with_parent = family_bic(&cov, n, 1, &[0], RIDGE, LAMBDA).unwrap();
    let no_parent = family_bic(&cov, n, 1, &[], RIDGE, LAMBDA).unwrap();
    assert!(
        with_parent > no_parent,
        "parent {with_parent} should beat empty {no_parent}"
    );
}

#[test]
fn near_singular_parent_block_stays_finite_via_ridge() {
    // Two collinear parents (identical columns) make Σ_PP singular; the ridge
    // keeps the Schur-complement solve finite.
    let t = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let p = [10.0, 12.0, 9.0, 15.0, 11.0, 14.0];
    let data = data_from_columns(&[&t, &p, &p]); // columns 1 and 2 identical
    let cov = data.sample_covariance().unwrap();

    let score = family_bic(&cov, t.len(), 0, &[1, 2], RIDGE, LAMBDA).unwrap();
    assert!(score.is_finite(), "ridge should keep the score finite");
}

#[test]
fn zero_variance_column_is_floored_not_infinite() {
    // A constant column has zero marginal variance; the floor keeps ln(σ²)
    // finite instead of producing −∞ / NaN.
    let constant = [5.0, 5.0, 5.0, 5.0, 5.0, 5.0];
    let other = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let data = data_from_columns(&[&constant, &other]);
    let cov = data.sample_covariance().unwrap();

    let score = family_bic(&cov, constant.len(), 0, &[], RIDGE, LAMBDA).unwrap();
    assert!(score.is_finite() && !score.is_nan(), "got {score}");
}

#[test]
fn node_index_out_of_bounds_is_an_error() {
    let (data, n) = xy_data();
    let cov = data.sample_covariance().unwrap(); // 2 × 2
    let err = family_bic(&cov, n, 2, &[], RIDGE, LAMBDA).unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::NodeOutOfBounds);
}

#[test]
fn parent_index_out_of_bounds_is_an_error() {
    let (data, n) = xy_data();
    let cov = data.sample_covariance().unwrap(); // 2 × 2
    let err = family_bic(&cov, n, 0, &[2], RIDGE, LAMBDA).unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::NodeOutOfBounds);
}

#[test]
fn non_square_covariance_is_a_dimension_mismatch() {
    let cov_bad = CausalTensor::from_slice(&[1.0_f64; 6], &[2, 3]);
    let err = family_bic(&cov_bad, 8, 0, &[], RIDGE, LAMBDA).unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::DimensionMismatch);
}

// --- BicScorer ------------------------------------------------------------

#[test]
fn scorer_reports_num_vars_and_delegates_to_family_bic() {
    let (data, n) = xy_data();
    let cov = data.sample_covariance().unwrap();
    let cfg = BossConfig::<f64>::with_seed(0);
    let scorer = BicScorer::new(&cov, n, &cfg).unwrap();

    assert_eq!(scorer.num_vars(), 2);
    let via_scorer = scorer.score(1, &[0]).unwrap();
    let via_free = family_bic(&cov, n, 1, &[0], cfg.ridge_eps, cfg.bic_lambda).unwrap();
    approx(via_scorer, via_free);
}

#[test]
fn scorer_propagates_index_errors() {
    let (data, n) = xy_data();
    let cov = data.sample_covariance().unwrap();
    let cfg = BossConfig::<f64>::default();
    let scorer = BicScorer::new(&cov, n, &cfg).unwrap();
    let err = scorer.score(5, &[]).unwrap_err();
    assert_eq!(*err.kind(), BrcdErrorEnum::NodeOutOfBounds);
}

#[test]
fn scorer_rejects_non_square_covariance() {
    // `BicScorer` borrows the covariance, so it is not `Debug`; match the error
    // rather than `unwrap_err`.
    let cov_bad = CausalTensor::from_slice(&[1.0_f64; 6], &[2, 3]);
    let cfg = BossConfig::<f64>::default();
    match BicScorer::new(&cov_bad, 8, &cfg) {
        Err(e) => assert_eq!(*e.kind(), BrcdErrorEnum::DimensionMismatch),
        Ok(_) => panic!("expected a dimension mismatch"),
    }
}

#[test]
fn scorer_rejects_one_dimensional_tensor() {
    let cov_bad = CausalTensor::from_slice(&[1.0_f64; 4], &[4]);
    let cfg = BossConfig::<f64>::default();
    match BicScorer::new(&cov_bad, 8, &cfg) {
        Err(e) => assert_eq!(*e.kind(), BrcdErrorEnum::DimensionMismatch),
        Ok(_) => panic!("expected a dimension mismatch"),
    }
}

#[test]
fn scorer_rejects_zero_samples() {
    let (data, _) = xy_data();
    let cov = data.sample_covariance().unwrap();
    let cfg = BossConfig::<f64>::default();
    match BicScorer::new(&cov, 0, &cfg) {
        Err(e) => assert_eq!(*e.kind(), BrcdErrorEnum::EmptyData),
        Ok(_) => panic!("expected empty-data error"),
    }
}
