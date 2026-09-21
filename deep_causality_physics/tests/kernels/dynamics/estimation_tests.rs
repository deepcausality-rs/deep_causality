/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, kalman_filter_linear_kernel};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_kalman_filter_linear_kernel_identity() {
    // 1D Kalman filter test with proper 2D tensor shapes for matmul
    // State x = [[10]] (1x1 column vector)
    // P = [[1]] (1x1 covariance matrix)
    // Measurement z = [[12]] (1x1)
    // H = [[1]] (1x1 measurement matrix)
    // R = [[1]] (1x1 measurement noise)

    // All tensors must be 2D for matmul operations
    let x_pred = CausalTensor::new(vec![10.0], vec![1, 1]).unwrap();
    let p_pred = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let measurement = CausalTensor::new(vec![12.0], vec![1, 1]).unwrap();
    let h = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap(); // Measurement matrix
    let r = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap(); // Measurement noise
    let q = CausalTensor::new(vec![0.0], vec![1, 1]).unwrap(); // Process noise (unused)

    let result = kalman_filter_linear_kernel::<f64>(&x_pred, &p_pred, &measurement, &h, &r, &q);

    assert!(result.is_ok(), "Kalman filter failed: {:?}", result.err());

    let (x_new, p_new) = result.unwrap();

    // Manual Calc:
    // y = z - Hx = 12 - 10 = 2
    // S = HPH' + R = 1*1*1 + 1 = 2
    // K = PH'S^-1 = 1*1*0.5 = 0.5
    // x_new = x + Ky = 10 + 0.5*2 = 11
    // P_new = (I - KH)P = (1 - 0.5*1)*1 = 0.5

    assert!(
        (x_new.data()[0] - 11.0).abs() < 1e-10,
        "Expected new state 11.0, got {:?}",
        x_new.data()[0]
    );
    assert!(
        (p_new.data()[0] - 0.5).abs() < 1e-10,
        "Expected new cov 0.5, got {:?}",
        p_new.data()[0]
    );
}

#[test]
fn test_kalman_filter_singular_error() {
    // Test case where S is non-invertible (S=0).
    // This happens if R=0 and P=0, or specific cancellations.
    // Let P=0, R=0, H=1
    // Then S = 0 + 0 = 0. Inverse should fail.

    let x_pred = CausalTensor::new(vec![10.0], vec![1, 1]).unwrap();
    let p_pred = CausalTensor::new(vec![0.0], vec![1, 1]).unwrap();
    let measurement = CausalTensor::new(vec![12.0], vec![1, 1]).unwrap();
    let h = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let r = CausalTensor::new(vec![0.0], vec![1, 1]).unwrap(); // Zero noise -> singularity
    let q = CausalTensor::new(vec![0.0], vec![1, 1]).unwrap();

    let result = kalman_filter_linear_kernel::<f64>(&x_pred, &p_pred, &measurement, &h, &r, &q);

    // Attempting to invert singular matrix S should return error
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::Singularity { .. }
        ),
        "expected a Singularity refusal"
    );
}

#[test]
fn test_kalman_filter_innovation_covariance_shape_mismatch() {
    // Line 76: if hph_t.shape() != measurement_noise.shape()
    // hph_t shape depends on H (MxN) and P (NxN). H*P*H^T -> MxM.
    // If we make R have wrong shape, it triggers this.
    // Let H be 1x1, P be 1x1. Then HPH' is 1x1.
    // Set R to 2x2.

    let x_pred = CausalTensor::new(vec![10.0], vec![1, 1]).unwrap();
    let p_pred = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let measurement = CausalTensor::new(vec![12.0], vec![1, 1]).unwrap();
    let h = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    // R is 2x2, but HPH' is 1x1
    let r = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();
    let q = CausalTensor::new(vec![0.0], vec![1, 1]).unwrap();

    let result = kalman_filter_linear_kernel::<f64>(&x_pred, &p_pred, &measurement, &h, &r, &q);

    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
    let err = result.unwrap_err();
    match err.0 {
        deep_causality_physics::PhysicsErrorEnum::DimensionMismatch(msg) => {
            assert!(msg.contains("Innovation covariance shape"));
        }
        _ => panic!("Expected DimensionMismatch, got {:?}", err),
    }
}

// `test_kalman_filter_state_update_shape_mismatch` and
// `test_kalman_filter_identity_shape_mismatch` were removed here. Both were named for a late
// shape guard, both conceded in their own comments that the guard cannot be reached, and both
// then asserted a bare `is_err()` satisfied by an unrelated earlier failure.
// `estimation_coverage_tests.rs` proves those two guards unreachable and covers the reachable
// path, so the pair added no discrimination.

#[test]
fn test_master_equation_zero_state_and_empty_history_stays_zero() {
    use deep_causality_physics::Probability;
    use deep_causality_physics::generalized_master_equation_kernel;

    let state = vec![Probability::<f64>::new(0.0).unwrap()];
    let history: Vec<Vec<Probability<f64>>> = vec![];
    let mk: Vec<CausalTensor<f64>> = vec![];

    let out = generalized_master_equation_kernel(&state, &history, None, &mk).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].value(), 0.0);
}

#[test]
fn test_master_equation_markov_limit_is_the_transition_matrix_applied_once() {
    use deep_causality_physics::Probability;
    use deep_causality_physics::generalized_master_equation_kernel;

    // P = 0.5, T = 0.8, no memory: result is T P = 0.4.
    let state = vec![Probability::<f64>::new(0.5).unwrap()];
    let history: Vec<Vec<Probability<f64>>> = vec![];
    let mk: Vec<CausalTensor<f64>> = vec![];
    let t = CausalTensor::new(vec![0.8], vec![1, 1]).unwrap();

    let out = generalized_master_equation_kernel(&state, &history, Some(&t), &mk).unwrap();
    assert!(
        (out[0].value() - 0.4).abs() < 1e-10,
        "got {}",
        out[0].value()
    );
}

#[test]
fn test_master_equation_memory_kernel_alone_contributes_its_convolution() {
    use deep_causality_physics::Probability;
    use deep_causality_physics::generalized_master_equation_kernel;

    // History = 0.5, K = 0.1, no transition matrix: result is K * hist = 0.05.
    let state = vec![Probability::<f64>::new(0.0).unwrap()];
    let history = vec![vec![Probability::<f64>::new(0.5).unwrap()]];
    let mk = vec![CausalTensor::new(vec![0.1], vec![1, 1]).unwrap()];

    let out = generalized_master_equation_kernel(&state, &history, None, &mk).unwrap();
    assert!(
        (out[0].value() - 0.05).abs() < 1e-10,
        "got {}",
        out[0].value()
    );
}

#[test]
fn test_master_equation_sums_the_markov_and_memory_contributions() {
    use deep_causality_physics::Probability;
    use deep_causality_physics::generalized_master_equation_kernel;

    // 0.4 from the Markov term and 0.05 from the memory term add to 0.45. The three cases are
    // separate so that a failure in the Markov term cannot hide the memory term.
    let state = vec![Probability::<f64>::new(0.5).unwrap()];
    let history = vec![vec![Probability::<f64>::new(0.5).unwrap()]];
    let mk = vec![CausalTensor::new(vec![0.1], vec![1, 1]).unwrap()];
    let t = CausalTensor::new(vec![0.8], vec![1, 1]).unwrap();

    let out = generalized_master_equation_kernel(&state, &history, Some(&t), &mk).unwrap();
    assert!(
        (out[0].value() - 0.45).abs() < 1e-10,
        "got {}",
        out[0].value()
    );
}

#[test]
fn test_master_equation_rejects_a_history_shorter_than_the_memory_kernel() {
    use deep_causality_physics::Probability;
    use deep_causality_physics::generalized_master_equation_kernel;

    let state = vec![Probability::<f64>::new(0.5).unwrap()];
    let history: Vec<Vec<Probability<f64>>> = vec![];
    let mk = vec![CausalTensor::new(vec![0.1], vec![1, 1]).unwrap()];

    assert!(
        matches!(
            generalized_master_equation_kernel(&state, &history, None, &mk)
                .unwrap_err()
                .0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_master_equation_rejects_a_history_entry_of_the_wrong_dimension() {
    use deep_causality_physics::Probability;
    use deep_causality_physics::generalized_master_equation_kernel;

    let state = vec![Probability::<f64>::new(0.5).unwrap()];
    let history = vec![vec![
        Probability::<f64>::new(0.5).unwrap(),
        Probability::<f64>::new(0.5).unwrap(),
    ]];
    let mk = vec![CausalTensor::new(vec![0.1], vec![1, 1]).unwrap()];

    assert!(
        matches!(
            generalized_master_equation_kernel(&state, &history, None, &mk)
                .unwrap_err()
                .0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}
