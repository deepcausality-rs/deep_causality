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
fn test_time_dilation_angle_metric_mismatch() {
    let t1 = CausalMultiVector::new(vec![0.0; 16], Metric::Minkowski(4)).unwrap();
    let t2 = CausalMultiVector::new(vec![0.0; 8], Metric::Euclidean(3)).unwrap();
    let result = time_dilation_angle_kernel(&t1, &t2);
    assert!(result.is_err());
}

#[test]
fn test_time_dilation_angle_not_minkowski() {
    let t1 = CausalMultiVector::new(vec![1.0, 0.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let t2 = CausalMultiVector::new(vec![1.0, 0.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let result = time_dilation_angle_kernel(&t1, &t2);
    assert!(result.is_err());
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

#[test]
fn test_chronometric_volume_kernel_metric_mismatch() {
    let a = CausalMultiVector::new(vec![0.0; 16], Metric::Minkowski(4)).unwrap();
    let b = CausalMultiVector::new(vec![0.0; 16], Metric::Minkowski(4)).unwrap();
    let c = CausalMultiVector::new(vec![0.0; 8], Metric::Euclidean(3)).unwrap();
    let result = chronometric_volume_kernel(&a, &b, &c);
    assert!(result.is_err());
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
    let result =
        deep_causality_physics::generate_schwarzschild_metric(-1.0 + 1e-10, 1.0, 1.0, 1.0).unwrap();
    let val = result.data()[0];
    assert!((val - (-1.0 + 1e-10)).abs() < 1e-15);
}

// =============================================================================
// parallel_transport_kernel Tests
// =============================================================================

use deep_causality_physics::{parallel_transport_kernel, proper_time_kernel};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_parallel_transport_flat_space() {
    // In flat space (Γ = 0), parallel transport preserves the vector
    let initial_vector = vec![1.0, 0.0, 0.0, 0.0];
    let path = vec![
        vec![0.0, 0.0, 0.0, 0.0],
        vec![1.0, 0.0, 0.0, 0.0],
        vec![2.0, 0.0, 0.0, 0.0],
    ];
    let christoffel = CausalTensor::new(vec![0.0; 64], vec![4, 4, 4]).unwrap();

    let result = parallel_transport_kernel(&initial_vector, &path, &christoffel);
    assert!(result.is_ok());

    let final_vector = result.unwrap();
    assert!((final_vector[0] - 1.0).abs() < 1e-10);
    assert!((final_vector[1] - 0.0).abs() < 1e-10);
}

#[test]
fn test_parallel_transport_short_path_error() {
    // Path with only 1 point should error
    let initial_vector = vec![1.0, 0.0];
    let path = vec![vec![0.0, 0.0]];
    let christoffel = CausalTensor::new(vec![0.0; 8], vec![2, 2, 2]).unwrap();

    let result = parallel_transport_kernel(&initial_vector, &path, &christoffel);
    assert!(result.is_err());
}

#[test]
fn test_parallel_transport_dimension_mismatch() {
    let initial_vector = vec![1.0, 0.0, 0.0]; // 3D
    let path = vec![vec![0.0, 0.0], vec![1.0, 0.0]]; // 2D path points
    let christoffel = CausalTensor::new(vec![0.0; 27], vec![3, 3, 3]).unwrap();

    let result = parallel_transport_kernel(&initial_vector, &path, &christoffel);
    assert!(result.is_err());
}

#[test]
fn test_parallel_transport_wrong_christoffel_rank() {
    let initial_vector = vec![1.0, 0.0];
    let path = vec![vec![0.0, 0.0], vec![1.0, 0.0]];
    let christoffel = CausalTensor::new(vec![0.0; 4], vec![2, 2]).unwrap(); // Rank 2

    let result = parallel_transport_kernel(&initial_vector, &path, &christoffel);
    assert!(result.is_err());
}

#[test]
fn test_parallel_transport_multiple_segments() {
    // Test with multiple path segments in flat space
    let initial_vector = vec![1.0, 2.0];
    let path = vec![
        vec![0.0, 0.0],
        vec![1.0, 0.0],
        vec![1.0, 1.0],
        vec![0.0, 1.0],
        vec![0.0, 0.0], // Back to start
    ];
    let christoffel = CausalTensor::new(vec![0.0; 8], vec![2, 2, 2]).unwrap();

    let result = parallel_transport_kernel(&initial_vector, &path, &christoffel);
    assert!(result.is_ok());

    // In flat space, vector should be unchanged
    let final_vector = result.unwrap();
    assert!((final_vector[0] - initial_vector[0]).abs() < 1e-10);
    assert!((final_vector[1] - initial_vector[1]).abs() < 1e-10);
}

// =============================================================================
// proper_time_kernel Tests
// =============================================================================

#[test]
fn test_proper_time_flat_minkowski() {
    // Timelike path in flat Minkowski space
    // Path along time axis: (t, 0, 0, 0) -> (t+1, 0, 0, 0)
    let path = vec![
        vec![0.0, 0.0, 0.0, 0.0],
        vec![1.0, 0.0, 0.0, 0.0],
        vec![2.0, 0.0, 0.0, 0.0],
    ];
    // Minkowski metric: diag(-1, 1, 1, 1)
    let metric = CausalTensor::new(
        vec![
            -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
        vec![4, 4],
    )
    .unwrap();

    let result = proper_time_kernel(&path, &metric);
    assert!(result.is_ok());

    let tau = result.unwrap();
    // ds² = -dt² for purely timelike motion
    // |ds²| = 1 for each step, so dτ = 1 per step, total = 2
    assert!((tau - 2.0).abs() < 1e-10, "Expected τ = 2.0, got {}", tau);
}

#[test]
fn test_proper_time_empty_path() {
    let path: Vec<Vec<f64>> = vec![];
    let metric = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();

    let result = proper_time_kernel(&path, &metric);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0.0);
}

#[test]
fn test_proper_time_single_point() {
    let path = vec![vec![0.0, 0.0]];
    let metric = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();

    let result = proper_time_kernel(&path, &metric);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0.0);
}

#[test]
fn test_proper_time_spacelike_path() {
    // Spacelike path in Euclidean metric
    let path = vec![vec![0.0, 0.0], vec![3.0, 4.0]]; // Distance = 5
    let metric = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();

    let result = proper_time_kernel(&path, &metric);
    assert!(result.is_ok());

    let tau = result.unwrap();
    assert!((tau - 5.0).abs() < 1e-10, "Expected τ = 5.0, got {}", tau);
}

#[test]
fn test_proper_time_wrong_metric_rank() {
    let path = vec![vec![0.0, 0.0], vec![1.0, 0.0]];
    let metric = CausalTensor::new(vec![1.0, 0.0], vec![2]).unwrap(); // Rank 1

    let result = proper_time_kernel(&path, &metric);
    assert!(result.is_err());
}

#[test]
fn test_proper_time_dimension_mismatch() {
    let path = vec![vec![0.0, 0.0, 0.0], vec![1.0, 0.0, 0.0]]; // 3D
    let metric = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap(); // 2D

    let result = proper_time_kernel(&path, &metric);
    assert!(result.is_err());
}
