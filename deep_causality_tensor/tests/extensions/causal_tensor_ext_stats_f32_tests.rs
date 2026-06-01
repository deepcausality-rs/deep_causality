/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Precision mirror of the stats-extension tests at `f32`, confirming the
//! generic implementation composes at single precision.

use deep_causality_tensor::{CausalTensor, CausalTensorStatsExt};

const EPS: f32 = 1e-5;

#[test]
fn sample_mean_at_f32() {
    let t = CausalTensor::new(vec![1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0], vec![3, 2]).unwrap();
    let means = t.sample_mean().expect("mean computes at f32");
    assert!((means.as_slice()[0] - 3.0).abs() < EPS);
    assert!((means.as_slice()[1] - 4.0).abs() < EPS);
}

#[test]
fn sample_covariance_at_f32() {
    let t =
        CausalTensor::new(vec![1.0_f32, 2.0, 2.0, 1.0, 3.0, 4.0, 4.0, 3.0], vec![4, 2]).unwrap();
    let cov = t.sample_covariance().expect("covariance computes at f32");
    let five_thirds = 5.0_f32 / 3.0;
    assert!((cov.get(&[0, 0]).unwrap() - five_thirds).abs() < EPS);
    assert!((cov.get(&[0, 1]).unwrap() - 1.0).abs() < EPS);
}

#[test]
fn logsumexp_at_f32() {
    let t = CausalTensor::new(vec![0.0_f32, 0.0, 0.0], vec![3, 1]).unwrap();
    assert!((t.logsumexp() - 3.0_f32.ln()).abs() < EPS);
}

#[test]
fn gaussian_log_density_at_f32() {
    use std::f32::consts::PI;
    let t = CausalTensor::new(vec![0.0_f32], vec![1, 1]).unwrap();
    let dens = t.gaussian_log_density(0.0, 1.0).unwrap();
    let expected = -0.5 * (2.0 * PI).ln();
    assert!((dens.as_slice()[0] - expected).abs() < EPS);
}

#[test]
fn conditional_variance_at_f32() {
    let cov = CausalTensor::new(vec![2.0_f32, 1.0, 1.0, 1.0], vec![2, 2]).unwrap();
    let cv = cov.conditional_variance(0, &[1], 0.0).unwrap();
    assert!((cv - 1.0).abs() < EPS);
}
