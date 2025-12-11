/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    chronometric_volume_kernel, spacetime_interval_kernel, time_dilation_angle_kernel,
};

// =============================================================================
// spacetime_interval_kernel Tests
// =============================================================================

#[test]
fn test_spacetime_interval_kernel_valid() {
    // Minkowski 4D vector: [t, x, y, z] with 16 components for 4D GA
    let mv = CausalMultiVector::new(
        vec![
            0.0, 5.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
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
        vec![
            0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Minkowski(4),
    )
    .unwrap();
    let t2 = CausalMultiVector::new(
        vec![
            0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
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
        vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Minkowski(4),
    )
    .unwrap();
    let t2 = CausalMultiVector::new(
        vec![
            0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Minkowski(4),
    )
    .unwrap();

    let result = time_dilation_angle_kernel(&t1, &t2);
    assert!(result.is_err(), "Zero magnitude should error");
}

#[test]
fn test_time_dilation_angle_causality_violation() {
    // Test CausalityViolation where gamma < 1.0.
    // This occurs if vectors are in opposite light cones (one future, one past),
    // resulting in negative dot product for timelike vectors in (+---) metric.
    // Or if they are spacelike separated in a way that violates assumptions.

    // Future pointing:  [0.0, 1.0, 0.0, ...] (assuming index 1 is Time per other tests logic or Scalar + Time?)
    // Wait, the previous parallel test used index 1=1.0 and index 1=2.0.
    // Let's create two opposing vectors.

    // t1: [..., 1.0, ...]
    let mut data1 = vec![0.0; 16];
    data1[1] = 1.0;
    let t1 = CausalMultiVector::new(data1, Metric::Minkowski(4)).unwrap();

    // t2: [..., -1.0, ...] (Opposite direction)
    let mut data2 = vec![0.0; 16];
    data2[1] = -1.0;
    let t2 = CausalMultiVector::new(data2, Metric::Minkowski(4)).unwrap();

    let result = time_dilation_angle_kernel(&t1, &t2);

    // dot = -1. mag1 = 1. mag2 = 1. gamma = -1.
    // -1 < 1.0 -> Error.
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err.0 {
        deep_causality_physics::PhysicsErrorEnum::CausalityViolation(msg) => {
            assert!(msg.contains("Invalid Lorentz factor"));
        }
        _ => panic!("Expected CausalityViolation, got {:?}", err),
    }
}

// =============================================================================
// chronometric_volume_kernel Tests
// =============================================================================

#[test]
fn test_chronometric_volume_kernel_valid() {
    let a = CausalMultiVector::new(
        vec![
            0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Minkowski(4),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![
            0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Minkowski(4),
    )
    .unwrap();
    let c = CausalMultiVector::new(
        vec![
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Minkowski(4),
    )
    .unwrap();

    let result = chronometric_volume_kernel(&a, &b, &c);
    assert!(result.is_ok());
}

// =============================================================================
// generate_schwarzschild_metric Tests
// =============================================================================

#[test]
fn test_schwarzschild_metric_success() {
    let result = deep_causality_physics::generate_schwarzschild_metric(-0.5, 2.0, 10.0, 5.0);
    assert!(result.is_ok());
    
    let metric = result.unwrap();
    let shape = metric.shape();
    assert_eq!(shape, vec![4, 4]);
    
    let data = metric.data();
    // Check diagonal elements
    // Index 0 (0,0) = g_00 = -0.5
    assert!((data[0] - (-0.5)).abs() < 1e-9);
    // Index 5 (1,1) = g_11 = 2.0
    assert!((data[5] - 2.0).abs() < 1e-9);
    // Index 10 (2,2) = g_22 = 10.0
    assert!((data[10] - 10.0).abs() < 1e-9);
    // Index 15 (3,3) = g_33 = 5.0
    assert!((data[15] - 5.0).abs() < 1e-9);
    
    // Check off-diagonal (e.g., index 1)
    assert_eq!(data[1], 0.0);
}

#[test]
fn test_schwarzschild_metric_values_check() {
    // Ensure that inputs are exactly propagated
    let result = deep_causality_physics::generate_schwarzschild_metric(
        -1.0 + 1e-10, 
        1.0, 
        1.0, 
        1.0
    ).unwrap();
    let val = result.data()[0];
    assert!((val - (-1.0 + 1e-10)).abs() < 1e-15);
}
