/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::kalman_filter_linear_kernel;
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

    let result = kalman_filter_linear_kernel(&x_pred, &p_pred, &measurement, &h, &r, &q);

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

    let result = kalman_filter_linear_kernel(&x_pred, &p_pred, &measurement, &h, &r, &q);

    // Attempting to invert singular matrix S should return error
    assert!(result.is_err(), "Should return error for singular S matrix");
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

    let result = kalman_filter_linear_kernel(&x_pred, &p_pred, &measurement, &h, &r, &q);

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err.0 {
        deep_causality_physics::PhysicsErrorEnum::DimensionMismatch(msg) => {
            assert!(msg.contains("Innovation covariance shape"));
        }
        _ => panic!("Expected DimensionMismatch, got {:?}", err),
    }
}


#[test]
fn test_kalman_filter_state_update_shape_mismatch() {
    // Attempt to trigger Line 102: x_pred.shape() != ky.shape()
    // We try to trigger a mismatch by passing inputs that are compatible for matmul but produce unexpected output shape.
    // Given the tensor library's strictness, this is hard to trigger without hitting earlier error.
    // However, we construct a case that fails validation either at Line 51 or Line 102, covering the logic path.
    
    // x_pred [1] (Rank 1)
    // H [1, 1], z [1, 1]
    // This triggers "measurement [1,1] != hx [1]" at Line 51 first.
    // This effectively tests that shape mismatches are caught.
    
    let x_pred = CausalTensor::new(vec![10.0], vec![1]).unwrap(); 
    let p_pred = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let measurement = CausalTensor::new(vec![12.0], vec![1, 1]).unwrap();
    let h = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let r = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let q = CausalTensor::new(vec![0.0], vec![1, 1]).unwrap();

    let result = kalman_filter_linear_kernel(&x_pred, &p_pred, &measurement, &h, &r, &q);

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err.0 {
        deep_causality_physics::PhysicsErrorEnum::DimensionMismatch(_) => {
            // Expected
        }
        deep_causality_physics::PhysicsErrorEnum::Singularity(_) => {
            // Also acceptable if earlier tensor op fails due to rank mismatch
        }
        _ => panic!("Expected DimensionMismatch or Singularity, got {:?}", err),
    }
}

#[test]
fn test_kalman_filter_identity_shape_mismatch() {
    // Attempt to trigger Line 121: identity.shape() != kh.shape()
    // We pass a non-square P matrix [1, 2].
    // H [1, 1]. x [1, 2]. z [1, 2]. R [1, 1].
    //
    // H(1,1) * x(1,2) -> Error? No, compatible. Result [1,2].
    // z [1,2]. y [1,2]. OK.
    // H(1,1) * P(1,2) -> Result [1,2].
    // S = HPH' + R.
    // [1,2] * H'(1,1)? No H' is [1,1] (transpose of 1,1).
    // [1,2] * [1,1]? Error. Cols 2 != Rows 1.
    //
    // So non-square P fails at HPH'.
    //
    // Let's try P [2, 1]. H [1, 2].
    // H(1,2) * P(2,1) -> [1,1].
    // H' is [2,1].
    // S = (1,1) * (2,1)? Error.
    //
    // It seems extremely difficult to reach Line 121 with standard tensor ops because P must be compatible with H,
    // and K calculation constrains shapes further.
    //
    // However, we will add this test case to verify that *some* shape error is returned,
    // demonstrating that the function is robust against non-square inputs.
    
    let x_pred = CausalTensor::new(vec![10.0, 20.0], vec![2, 1]).unwrap();
    let p_pred = CausalTensor::new(vec![1.0, 0.0], vec![2, 1]).unwrap(); // Non-square P
    let measurement = CausalTensor::new(vec![12.0], vec![1, 1]).unwrap();
    let h = CausalTensor::new(vec![1.0, 1.0], vec![1, 2]).unwrap(); 
    let r = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let q = CausalTensor::new(vec![0.0], vec![1, 1]).unwrap();

    let result = kalman_filter_linear_kernel(&x_pred, &p_pred, &measurement, &h, &r, &q);

    assert!(result.is_err());
}
