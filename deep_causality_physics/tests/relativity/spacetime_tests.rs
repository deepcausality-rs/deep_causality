/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{chronometric_volume_kernel, spacetime_interval_kernel, time_dilation_angle_kernel};

// =============================================================================
// spacetime_interval_kernel Tests
// =============================================================================

#[test]
fn test_spacetime_interval_kernel_valid() {
    // Minkowski 4D vector: [t, x, y, z] with 16 components for 4D GA
    let mv = CausalMultiVector::new(
        vec![0.0, 5.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Minkowski(4),
    )
    .unwrap();
    let metric = Metric::Minkowski(4);
    
    let result = spacetime_interval_kernel(&mv, &metric);
    assert!(result.is_ok());
}

#[test]
fn test_spacetime_interval_kernel_metric_mismatch() {
    let mv = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let metric = Metric::Minkowski(4);
    
    let result = spacetime_interval_kernel(&mv, &metric);
    assert!(result.is_err(), "Should error on metric mismatch");
}

// =============================================================================
// time_dilation_angle_kernel Tests
// =============================================================================

#[test]
fn test_time_dilation_angle_parallel_vectors() {
    // Two parallel timelike vectors should have zero rapidity
    let t1 = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Minkowski(4),
    )
    .unwrap();
    let t2 = CausalMultiVector::new(
        vec![0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Minkowski(4),
    )
    .unwrap();
    
    let result = time_dilation_angle_kernel(&t1, &t2);
    // Parallel vectors with gamma~1 => eta~0
    assert!(result.is_ok());
}

#[test]
fn test_time_dilation_angle_zero_magnitude_error() {
    let t1 = CausalMultiVector::new(
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Minkowski(4),
    )
    .unwrap();
    let t2 = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Minkowski(4),
    )
    .unwrap();
    
    let result = time_dilation_angle_kernel(&t1, &t2);
    assert!(result.is_err(), "Zero magnitude should error");
}

// =============================================================================
// chronometric_volume_kernel Tests
// =============================================================================

#[test]
fn test_chronometric_volume_kernel_valid() {
    let a = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Minkowski(4),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Minkowski(4),
    )
    .unwrap();
    let c = CausalMultiVector::new(
        vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Minkowski(4),
    )
    .unwrap();
    
    let result = chronometric_volume_kernel(&a, &b, &c);
    assert!(result.is_ok());
}
