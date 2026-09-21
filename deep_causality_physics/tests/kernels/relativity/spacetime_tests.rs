/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    PhysicsErrorEnum, chronometric_volume_kernel, spacetime_interval_kernel,
    time_dilation_angle_kernel,
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
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::MetricSingularity { .. }
        ),
        "expected a MetricSingularity refusal"
    );
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
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::CausalityViolation { .. }
        ),
        "expected a CausalityViolation refusal"
    );
}

#[test]
fn test_time_dilation_angle_metric_mismatch() {
    let t1 = CausalMultiVector::new(vec![0.0; 16], Metric::Minkowski(4)).unwrap();
    let t2 = CausalMultiVector::new(vec![0.0; 8], Metric::Euclidean(3)).unwrap();
    let result = time_dilation_angle_kernel(&t1, &t2);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::MetricSingularity { .. }
        ),
        "expected a MetricSingularity refusal"
    );
}

#[test]
fn test_time_dilation_angle_not_minkowski() {
    let t1 = CausalMultiVector::new(vec![1.0, 0.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let t2 = CausalMultiVector::new(vec![1.0, 0.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let result = time_dilation_angle_kernel(&t1, &t2);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::MetricSingularity { .. }
        ),
        "expected a MetricSingularity refusal"
    );
}

#[test]
fn test_time_dilation_angle_causality_violation() {
    // Test CausalityViolation where gamma < 1.0.
    // This occurs if vectors are in opposite light cones (one future, one past),
    // resulting in negative dot product for timelike vectors in (+---) metric.
    // Or if they are spacelike separated in a way that violates assumptions.

    // Two timelike vectors pointing into opposite light cones: slot 1 carries +1 in one and -1
    // in the other, so their Minkowski product is negative and gamma falls below 1.
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
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::CausalityViolation { .. }
        ),
        "expected a CausalityViolation refusal"
    );
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
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::MetricSingularity { .. }
        ),
        "expected a MetricSingularity refusal"
    );
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
    assert!((data[0] - (-0.5f64)).abs() < 1e-9);
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
    assert!((val - (-1.0 + 1e-10f64)).abs() < 1e-15);
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
    assert!((final_vector[0] - 1.0f64).abs() < 1e-10);
    assert!((final_vector[1] - 0.0f64).abs() < 1e-10);
}

// The transport tests above all set Gamma = 0, where `dv = -Gamma dx v` vanishes identically and
// transport is the identity for any implementation. The three below give it a real connection.

#[test]
fn test_parallel_transport_against_hand_computed_schwarzschild_components() {
    // The connection comes from `schwarzschild_christoffel_at`, whose components are checked
    // against their closed forms in theories/general_relativity/metrics_tests.rs. At M = 1,
    // r = 10: f = 0.8, f' = 0.02, so Gamma^t_{rt} = f'/(2f) = 0.0125 and
    // Gamma^r_{rr} = -f'/(2f) = -0.0125.
    //
    // A single radial step dx = (0, h, 0, 0) contracts against exactly one of them at a time,
    // so the expected answer is one product rather than a sum:
    //   v = (1,0,0,0):  dv^t = -Gamma^t_{rt} h v^t = -0.0125 h
    //   v = (0,1,0,0):  dv^r = -Gamma^r_{rr} h v^r = +0.0125 h
    use deep_causality_physics::theories::general_relativity::schwarzschild_christoffel_at;

    let christoffel = schwarzschild_christoffel_at(1.0_f64, 10.0).expect("a valid connection");
    let h = 1.0e-3_f64;
    let path = vec![vec![0.0, 10.0, 0.0, 0.0], vec![0.0, 10.0 + h, 0.0, 0.0]];

    let timelike = parallel_transport_kernel(&[1.0, 0.0, 0.0, 0.0], &path, &christoffel).unwrap();
    assert!(
        (timelike[0] - (1.0 - 0.0125 * h)).abs() < 1e-15,
        "v^t = {}, expected {}",
        timelike[0],
        1.0 - 0.0125 * h
    );
    for (mu, component) in timelike.iter().enumerate().skip(1) {
        assert!(component.abs() < 1e-15, "v^{mu} = {component}");
    }

    let radial = parallel_transport_kernel(&[0.0, 1.0, 0.0, 0.0], &path, &christoffel).unwrap();
    assert!(
        (radial[1] - (1.0 + 0.0125 * h)).abs() < 1e-15,
        "v^r = {}, expected {}",
        radial[1],
        1.0 + 0.0125 * h
    );
    assert!(radial[0].abs() < 1e-15, "v^t = {}", radial[0]);
}

#[test]
fn test_parallel_transport_pins_the_lower_index_order() {
    // A real Christoffel symbol is symmetric in its lower indices, so a Schwarzschild fixture
    // cannot tell `Gamma^mu_{nu rho}` from `Gamma^mu_{rho nu}`. The kernel accepts any rank-3
    // tensor, so an asymmetric one settles it.
    //
    // Gamma^0_{1 0} = 1 and nothing else. With dx = (0, 1) and v = (1, 0):
    //   dv^0 = -Gamma^0_{1 0} dx^1 v^0 = -1   =>  v_new = (0, 0)
    // Reading the lower indices the other way round picks up Gamma^0_{0 1} = 0 and leaves
    // v_new = (1, 0).
    let mut gamma = vec![0.0f64; 8]; // [2, 2, 2]; index mu*4 + nu*2 + rho
    gamma[2] = 1.0; // mu = 0, nu = 1, rho = 0 -> 0*4 + 1*2 + 0
    let christoffel = CausalTensor::new(gamma, vec![2, 2, 2]).unwrap();
    let path = vec![vec![0.0, 0.0], vec![0.0, 1.0]];

    let v = parallel_transport_kernel(&[1.0, 0.0], &path, &christoffel).unwrap();
    assert!((v[0] - 0.0).abs() < 1e-15, "v^0 = {}", v[0]);
    assert!((v[1] - 0.0).abs() < 1e-15, "v^1 = {}", v[1]);
}

#[test]
fn test_parallel_transport_is_linear_in_the_initial_vector() {
    // `dv = -Gamma dx v` is linear in v, so transporting k*v must give k times the transport of
    // v. Holds for any connection and needs no oracle.
    use deep_causality_physics::theories::general_relativity::schwarzschild_christoffel_at;

    let christoffel = schwarzschild_christoffel_at(1.0_f64, 10.0).expect("a valid connection");
    let path = vec![
        vec![0.0, 10.0, 0.0, 0.0],
        vec![0.1, 10.5, 0.0, 0.0],
        vec![0.2, 11.0, 0.3, 0.0],
    ];
    let v0 = [1.0_f64, 0.5, -0.25, 2.0];

    let base = parallel_transport_kernel(&v0, &path, &christoffel).unwrap();
    for k in [0.5_f64, 2.0, -3.0] {
        let scaled_v0: Vec<f64> = v0.iter().map(|x| x * k).collect();
        let scaled = parallel_transport_kernel(&scaled_v0, &path, &christoffel).unwrap();
        for mu in 0..4 {
            assert!(
                (scaled[mu] - k * base[mu]).abs() < 1e-12,
                "k = {k}, component {mu}: {} vs {}",
                scaled[mu],
                k * base[mu]
            );
        }
    }
}

#[test]
fn test_parallel_transport_short_path_error() {
    // Path with only 1 point should error
    let initial_vector = vec![1.0, 0.0];
    let path = vec![vec![0.0, 0.0]];
    let christoffel = CausalTensor::new(vec![0.0; 8], vec![2, 2, 2]).unwrap();

    let result = parallel_transport_kernel(&initial_vector, &path, &christoffel);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_parallel_transport_dimension_mismatch() {
    let initial_vector = vec![1.0, 0.0, 0.0]; // 3D
    let path = vec![vec![0.0, 0.0], vec![1.0, 0.0]]; // 2D path points
    let christoffel = CausalTensor::new(vec![0.0; 27], vec![3, 3, 3]).unwrap();

    let result = parallel_transport_kernel(&initial_vector, &path, &christoffel);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_parallel_transport_wrong_christoffel_rank() {
    let initial_vector = vec![1.0, 0.0];
    let path = vec![vec![0.0, 0.0], vec![1.0, 0.0]];
    let christoffel = CausalTensor::new(vec![0.0; 4], vec![2, 2]).unwrap(); // Rank 2

    let result = parallel_transport_kernel(&initial_vector, &path, &christoffel);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
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
    let final_vector: Vec<f64> = result.unwrap();
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
    assert!(
        (tau - 2.0f64).abs() < 1e-10,
        "Expected τ = 2.0, got {}",
        tau
    );
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

    let tau: f64 = result.unwrap();
    assert!((tau - 5.0).abs() < 1e-10, "Expected τ = 5.0, got {}", tau);
}

#[test]
fn test_proper_time_wrong_metric_rank() {
    let path = vec![vec![0.0, 0.0], vec![1.0, 0.0]];
    let metric = CausalTensor::new(vec![1.0, 0.0], vec![2]).unwrap(); // Rank 1

    let result = proper_time_kernel(&path, &metric);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_proper_time_non_square_metric_error() {
    let path = vec![vec![0.0, 0.0], vec![1.0, 0.0]];
    // Square check uses shape[1] != shape[0]. Build a [2, 3] rank-2 tensor.
    let metric = CausalTensor::new(vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0], vec![2, 3]).unwrap();
    let r = proper_time_kernel(&path, &metric);
    assert!(
        matches!(
            r.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_proper_time_nan_segment_error() {
    // A NaN coordinate yields a non-finite proper-time increment.
    let path = vec![vec![0.0, 0.0], vec![f64::NAN, 0.0]];
    let metric = CausalTensor::new(vec![-1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();
    let r = proper_time_kernel(&path, &metric);
    assert!(
        matches!(
            r.as_ref().unwrap_err().0,
            PhysicsErrorEnum::NumericalInstability { .. }
        ),
        "expected a NumericalInstability refusal"
    );
}

#[test]
fn test_parallel_transport_nan_christoffel_error() {
    let initial_vector = vec![1.0, 0.0];
    let path = vec![vec![0.0, 0.0], vec![1.0, 0.0]];
    // Non-finite Christoffel produces non-finite transported vector.
    let mut data = vec![0.0; 8];
    data[0] = f64::NAN;
    let christoffel = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    let r = parallel_transport_kernel(&initial_vector, &path, &christoffel);
    assert!(
        matches!(
            r.as_ref().unwrap_err().0,
            PhysicsErrorEnum::NumericalInstability { .. }
        ),
        "expected a NumericalInstability refusal"
    );
}

#[test]
fn test_time_dilation_angle_gamma_clamp_near_one() {
    // Identical vectors → gamma=1.0 exactly, exercises the clamp/sqrt(0) branch.
    let mut data = vec![0.0; 16];
    data[1] = 1.0;
    let t = CausalMultiVector::new(data, Metric::Minkowski(4)).unwrap();
    let eta = time_dilation_angle_kernel(&t, &t).unwrap();
    let val: f64 = eta.value();
    assert!(
        val.abs() < 1e-10,
        "Identical vectors give eta=0, got {}",
        val
    );
}

#[test]
fn test_proper_time_dimension_mismatch() {
    let path = vec![vec![0.0, 0.0, 0.0], vec![1.0, 0.0, 0.0]]; // 3D
    let metric = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap(); // 2D

    let result = proper_time_kernel(&path, &metric);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_time_dilation_angle_non_scalar_grade_error() {
    // Craft t1 = e0 (grade-1) and t2 = e0∧e1 (grade-2 bivector). The left
    // contraction e0 ⌋ (e0∧e1) = ±e1 is grade-1, so the inner product carries a
    // non-zero non-scalar component, triggering the "did not yield scalar grade"
    // branch.
    //
    // Binary blade indexing (Minkowski(4)): e0→idx 1, e1→idx 2, e0∧e1→idx 3.
    let mut d1 = vec![0.0f64; 16];
    d1[1] = 1.0; // e0
    let t1 = CausalMultiVector::<f64>::new(d1, Metric::Minkowski(4)).unwrap();

    let mut d2 = vec![0.0f64; 16];
    d2[3] = 1.0; // e0 ∧ e1 (bivector)
    let t2 = CausalMultiVector::<f64>::new(d2, Metric::Minkowski(4)).unwrap();

    let result = time_dilation_angle_kernel(&t1, &t2);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    match result.unwrap_err().0 {
        deep_causality_physics::PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("scalar grade"), "unexpected message: {}", msg);
        }
        other => panic!("Expected PhysicalInvariantBroken, got {:?}", other),
    }
}
