/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Precision sweep: every stats primitive runs at `f32`, `f64`, and `Float106`
//! and produces the same (precision-tolerant) result. This is the Tier A
//! "no hardwired f64" guard — the generic body is instantiated once per type.

use deep_causality_num::{ConjugateScalar, Float106, FromPrimitive, RealField, ToPrimitive};
use deep_causality_tensor::{CausalTensor, CausalTensorStatsExt};
use std::f64::consts::PI;

/// Runs every stats primitive at precision `T` and checks each result against
/// its closed form (converted back to `f64` for a precision-tolerant compare).
fn run_sweep<T>()
where
    T: RealField + FromPrimitive + ConjugateScalar<Real = T> + ToPrimitive,
{
    let f = |v: f64| <T as FromPrimitive>::from_f64(v).expect("representable");
    let to_f64 = |x: T| x.to_f64().expect("convertible");
    // f32 carries the loosest tolerance; use one bound for all three.
    let tol = 1e-4;

    // sample_mean: columns [1,3,5] -> 3, [2,4,6] -> 4
    let data = CausalTensor::new(
        vec![f(1.0), f(2.0), f(3.0), f(4.0), f(5.0), f(6.0)],
        vec![3, 2],
    )
    .unwrap();
    let means = data.sample_mean().unwrap();
    assert!((to_f64(means.as_slice()[0]) - 3.0).abs() < tol);
    assert!((to_f64(means.as_slice()[1]) - 4.0).abs() < tol);

    // sample_covariance: diagonal variance = 4 for both columns
    let cov = data.sample_covariance().unwrap();
    assert!((to_f64(*cov.get(&[0, 0]).unwrap()) - 4.0).abs() < tol);
    assert!((to_f64(*cov.get(&[1, 1]).unwrap()) - 4.0).abs() < tol);

    // logsumexp of three zeros = ln(3)
    let zeros = CausalTensor::new(vec![f(0.0), f(0.0), f(0.0)], vec![3, 1]).unwrap();
    assert!((to_f64(zeros.logsumexp()) - 3.0_f64.ln()).abs() < tol);

    // gaussian_log_density of standard normal at 0 = -0.5*ln(2π)
    let point = CausalTensor::new(vec![f(0.0)], vec![1, 1]).unwrap();
    let dens = point.gaussian_log_density(f(0.0), f(1.0)).unwrap();
    let expected = -0.5 * (2.0 * PI).ln();
    assert!((to_f64(dens.as_slice()[0]) - expected).abs() < tol);

    // conditional_variance: Σ=[[2,1],[1,1]], Var(y|p) = 1
    let joint = CausalTensor::new(vec![f(2.0), f(1.0), f(1.0), f(1.0)], vec![2, 2]).unwrap();
    let cv = joint.conditional_variance(0, &[1], f(0.0)).unwrap();
    assert!((to_f64(cv) - 1.0).abs() < tol);
}

#[test]
fn stats_primitives_sweep_at_f32() {
    run_sweep::<f32>();
}

#[test]
fn stats_primitives_sweep_at_f64() {
    run_sweep::<f64>();
}

#[test]
fn stats_primitives_sweep_at_float106() {
    run_sweep::<Float106>();
}
