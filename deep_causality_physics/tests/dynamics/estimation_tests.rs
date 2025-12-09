/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::kalman_filter_linear_kernel;
use deep_causality_tensor::CausalTensor;

#[test]
fn test_kalman_filter_linear_kernel_identity() {
    // 1D Kalman filter test
    // State x = [10]
    // P = [1]
    // Measurement z = [12]
    // H = [1] (measure state directly)
    // R = [1] (measurement noise)
    
    let x_pred = CausalTensor::new(vec![10.0], vec![1]).unwrap();
    let p_pred = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let measurement = CausalTensor::new(vec![12.0], vec![1]).unwrap();
    let h = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap(); // Measurement matrix
    let r = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap(); // Measurement noise
    let q = CausalTensor::new(vec![0.0], vec![1, 1]).unwrap(); // Process noise (unused)

    let result = kalman_filter_linear_kernel(&x_pred, &p_pred, &measurement, &h, &r, &q);
    
    assert!(result.is_ok());
    
    let (x_new, p_new) = result.unwrap();
    
    // Manual Calc:
    // y = z - Hx = 12 - 10 = 2
    // S = HPH' + R = 1*1*1 + 1 = 2
    // K = PH'S^-1 = 1*1*0.5 = 0.5
    // x_new = x + Ky = 10 + 0.5*2 = 11
    // P_new = (I - KH)P = (1 - 0.5*1)*1 = 0.5
    
    assert!((x_new.data()[0] - 11.0).abs() < 1e-10, "Expected new state 11.0, got {:?}", x_new.data()[0]);
    assert!((p_new.data()[0] - 0.5).abs() < 1e-10, "Expected new cov 0.5, got {:?}", p_new.data()[0]);
}

#[test]
fn test_kalman_filter_singular_error() {
    // Test case where S is non-invertible (e.g. S=0). 
    // This happens if R=0 and P=0, or specific cancellations.
    // Let P=0, R=0, H=1
    // Then S = 0 + 0 = 0. Inverse should fail.
    
    let x_pred = CausalTensor::new(vec![10.0], vec![1]).unwrap();
    let p_pred = CausalTensor::new(vec![0.0], vec![1, 1]).unwrap();
    let measurement = CausalTensor::new(vec![12.0], vec![1]).unwrap();
    let h = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap(); 
    let r = CausalTensor::new(vec![0.0], vec![1, 1]).unwrap(); // Zero noise -> singularity potential
    let q = CausalTensor::new(vec![0.0], vec![1, 1]).unwrap();
    
    let result = kalman_filter_linear_kernel(&x_pred, &p_pred, &measurement, &h, &r, &q);
    
    // Attempting to invert singular matrix S should return error
    assert!(result.is_err(), "Should return error for singular S matrix");
}
