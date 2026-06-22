/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::lorentz_force_kernel;

// =============================================================================
// lorentz_force_kernel Tests (F = J × B)
// =============================================================================

#[test]
fn test_lorentz_force_kernel_valid() {
    // Current density J in x-direction
    let j = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    // Magnetic field B in y-direction
    let b = CausalMultiVector::new(
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = lorentz_force_kernel(&j, &b);
    assert!(result.is_ok());

    let force = result.unwrap();
    // Result is a bivector from outer product J ^ B
    assert!(!force.data().is_empty());
}

#[test]
fn test_lorentz_force_kernel_parallel_vectors() {
    // Parallel J and B should give zero force
    let j = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = lorentz_force_kernel(&j, &b);
    assert!(result.is_ok());
    // Outer product of parallel vectors is zero
}

#[test]
fn test_lorentz_force_kernel_zero_current() {
    let j = CausalMultiVector::new(
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = lorentz_force_kernel(&j, &b);
    assert!(result.is_ok());
    // Zero current gives zero force
}

#[test]
fn test_lorentz_force_kernel_zero_field() {
    let j = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = lorentz_force_kernel(&j, &b);
    assert!(result.is_ok());
    // Zero field gives zero force
}

#[test]
fn test_lorentz_force_kernel_metric_mismatch_error() {
    // Different metrics => DimensionMismatch error branch.
    let j = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();

    assert!(lorentz_force_kernel(&j, &b).is_err());
}

#[test]
fn test_lorentz_force_kernel_overflow_result_is_rejected() {
    // Inputs are finite, but the outer product (J ∧ B) of huge non-parallel
    // vectors overflows to ±inf, tripping the post-computation non-finite
    // guard at lines 40-43 (distinct from the metric-mismatch branch).
    let j = CausalMultiVector::new(
        vec![0.0, f64::MAX, f64::MAX, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![0.0, f64::MAX, 0.0, f64::MAX, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    assert!(lorentz_force_kernel(&j, &b).is_err());
}
