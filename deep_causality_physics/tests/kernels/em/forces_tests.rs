/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{PhysicsErrorEnum, lorentz_force_kernel};

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

    let force = lorentz_force_kernel(&j, &b).unwrap();

    // J ^ B = e1 ^ e2 = e12. Blades are indexed by bitmask, so e1 = 1, e2 = 2, e12 = 3.
    // Every other blade must be zero: `!data().is_empty()` cannot see any of this, because a
    // Cl(3) multivector always carries eight components whatever the kernel computed.
    let d: &[f64] = force.data();
    assert_eq!(d.len(), 8, "Cl(3) carries eight blades");
    assert!((d[3] - 1.0).abs() < 1e-12, "e12 component = {}", d[3]);
    for (i, v) in d.iter().enumerate() {
        if i != 3 {
            assert!(v.abs() < 1e-12, "blade {i} should vanish, got {v}");
        }
    }
}

#[test]
fn test_lorentz_force_is_antisymmetric_in_its_arguments() {
    // J ^ B = -(B ^ J) for any J and B. An implementation using the geometric or inner product
    // instead of the outer one fails this; no oracle is needed.
    let j = CausalMultiVector::new(
        vec![0.0, 1.0, 2.0, 0.0, 3.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![0.0, -0.5, 1.5, 0.0, 0.25, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let forward = lorentz_force_kernel(&j, &b).unwrap();
    let reversed = lorentz_force_kernel(&b, &j).unwrap();
    let fwd: &[f64] = forward.data();
    let rev: &[f64] = reversed.data();
    for (i, (f, r)) in fwd.iter().zip(rev).enumerate() {
        assert!(
            (f + r).abs() < 1e-12,
            "blade {i}: {f} and {r} are not negatives"
        );
    }
}

#[test]
fn test_lorentz_force_vanishes_when_a_vector_is_wedged_with_itself() {
    // J ^ J = 0 identically. This is the nilpotency of the outer product, and it fails for the
    // geometric product, whose scalar part is |J|^2.
    let j = CausalMultiVector::new(
        vec![0.0, 1.0, 2.0, 0.0, 3.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let f = lorentz_force_kernel(&j, &j).unwrap();
    let d: &[f64] = f.data();
    for (i, v) in d.iter().enumerate() {
        assert!(v.abs() < 1e-12, "J ^ J blade {i} = {v}, must be zero");
    }
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

    assert!(
        matches!(
            lorentz_force_kernel(&j, &b).unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
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

    assert!(
        matches!(
            lorentz_force_kernel(&j, &b).unwrap_err().0,
            PhysicsErrorEnum::NumericalInstability { .. }
        ),
        "expected a NumericalInstability refusal"
    );
}
