/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor}; // Import Tensor trait

/// Standard Linear Kalman Filter Update Step.
pub fn kalman_filter_linear(
    x_pred: &CausalTensor<f64>,
    p_pred: &CausalTensor<f64>,
    measurement: &CausalTensor<f64>,
    measurement_matrix: &CausalTensor<f64>,
    measurement_noise: &CausalTensor<f64>,
    _process_noise: &CausalTensor<f64>,
) -> PropagatingEffect<(CausalTensor<f64>, CausalTensor<f64>)> {
    // 1. Innovation (Residual): y = z - H * x
    // H * x
    let hx = match measurement_matrix.matmul(x_pred) {
        Ok(res) => res,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError::from(PhysicsError::from(e)));
        }
    };

    // y = z - hx
    // sub returns Self, panics on error.
    let y = measurement.sub(&hx);

    // 2. Innovation Covariance: S = H * P * H^T + R
    // H * P
    let hp = match measurement_matrix.matmul(p_pred) {
        Ok(res) => res,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError::from(PhysicsError::from(e)));
        }
    };

    // H^T
    let ht_op = EinSumOp::<f64>::transpose(measurement_matrix.clone(), vec![1, 0]);
    let ht = match CausalTensor::ein_sum(&ht_op) {
        Ok(res) => res,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError::from(PhysicsError::from(e)));
        }
    };

    // (H * P) * H^T
    let hph_t = match hp.matmul(&ht) {
        Ok(res) => res,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError::from(PhysicsError::from(e)));
        }
    };

    // S = ... + R
    // add returns Self
    let s = hph_t.add(measurement_noise);

    // 3. Optimal Kalman Gain: K = P * H^T * S^-1
    // S^-1
    // CausalTensor must implement inverse.
    let s_inv = match s.inverse() {
        Ok(res) => res,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError::from(PhysicsError::from(e)));
        }
    };

    // P * H^T
    let pht = match p_pred.matmul(&ht) {
        Ok(res) => res,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError::from(PhysicsError::from(e)));
        }
    };

    // K = ... * S^-1
    let k = match pht.matmul(&s_inv) {
        Ok(res) => res,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError::from(PhysicsError::from(e)));
        }
    };

    // 4. State Update: x_new = x + K * y
    // K * y
    let ky = match k.matmul(&y) {
        Ok(res) => res,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError::from(PhysicsError::from(e)));
        }
    };

    let x_new = x_pred.add(&ky);

    // 5. Covariance Update: P_new = (I - K * H) * P
    // K * H
    let kh = match k.matmul(measurement_matrix) {
        Ok(res) => res,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError::from(PhysicsError::from(e)));
        }
    };

    // I (Identity matrix matching P dimension)
    // CausalTensor::identity(shape).
    // Assuming P is square NxN.
    let shape = p_pred.shape();
    // Assuming 2D for covariance. check rank?
    // identity logic depends on tensor API.
    let identity = match CausalTensor::identity(shape) {
        Ok(res) => res,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError::from(PhysicsError::from(e)));
        }
    };

    // I - KH
    let i_kh = identity.sub(&kh);

    // ... * P
    let p_new = match i_kh.matmul(p_pred) {
        Ok(res) => res,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError::from(PhysicsError::from(e)));
        }
    };

    PropagatingEffect::pure((x_new, p_new))
}
