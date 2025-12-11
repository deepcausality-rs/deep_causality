/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};

/// Standard Linear Kalman Filter Update Step.
///
/// Implements the discrete-time Kalman filter update equations:
///
/// 1. Innovation Residual: $\mathbf{y} = \mathbf{z} - \mathbf{H}\hat{\mathbf{x}}$
/// 2. Innovation Covariance: $\mathbf{S} = \mathbf{H}\mathbf{P}\mathbf{H}^T + \mathbf{R}$
/// 3. Optimal Kalman Gain: $\mathbf{K} = \mathbf{P}\mathbf{H}^T \mathbf{S}^{-1}$
/// 4. State Update: $\hat{\mathbf{x}}_{new} = \hat{\mathbf{x}} + \mathbf{K}\mathbf{y}$
/// 5. Covariance Update: $\mathbf{P}_{new} = (\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}$
///
/// # Arguments
/// * `x_pred` - Predicted state vector ($\hat{\mathbf{x}}$).
/// * `p_pred` - Predicted estimate covariance ($\mathbf{P}$).
/// * `measurement` - Observation vector ($\mathbf{z}$).
/// * `measurement_matrix` - Observation model ($\mathbf{H}$).
/// * `measurement_noise` - Observation noise covariance ($\mathbf{R}$).
/// * `_process_noise` - Process noise covariance (unused in update step, typically used in prediction).
///
/// # Returns
/// * `PropagatingEffect<(CausalTensor<f64>, CausalTensor<f64>)>` - Tuple of (Updated State, Updated Covariance).
pub fn kalman_filter_linear_kernel(
    x_pred: &CausalTensor<f64>,
    p_pred: &CausalTensor<f64>,
    measurement: &CausalTensor<f64>,
    measurement_matrix: &CausalTensor<f64>,
    measurement_noise: &CausalTensor<f64>,
    process_noise: &CausalTensor<f64>,
) -> Result<(CausalTensor<f64>, CausalTensor<f64>), PhysicsError> {
    // 1. Innovation (Residual): y = z - H * x
    // H * x
    let hx = measurement_matrix
        .matmul(x_pred)
        .map_err(PhysicsError::from)?;

    // y = z - hx
    // sub returns Self, panics on error? No, CausalTensor usually returns Result if shapes mismatch, but here we assume validation or it panics?
    // Looking at previous code: "sub returns Self, panics on error."
    // If it panics, we can't catch it easily without checks.
    // Assuming tensor ops here are safe-ish or we accept panic for now if API designs it so.
    // The previous code didn't wrap .sub() in match, so it likely returns T or panics.
    // Wait, let's look at `measurement.sub(&hx)`.
    // Check shapes before subtraction
    if measurement.shape() != hx.shape() {
        return Err(PhysicsError::new(
            crate::PhysicsErrorEnum::DimensionMismatch(format!(
                "Measurement shape {:?} != prediction shape {:?}",
                measurement.shape(),
                hx.shape()
            )),
        ));
    }
    let y = measurement.sub(&hx);

    // 2. Innovation Covariance: S = H * P * H^T + R
    // H * P
    let hp = measurement_matrix
        .matmul(p_pred)
        .map_err(PhysicsError::from)?;

    // H^T
    let ht_op = EinSumOp::<f64>::transpose(measurement_matrix.clone(), vec![1, 0]);
    let ht = CausalTensor::ein_sum(&ht_op).map_err(PhysicsError::from)?;

    // (H * P) * H^T
    let hph_t = hp.matmul(&ht).map_err(PhysicsError::from)?;

    // S = ... + R
    if hph_t.shape() != measurement_noise.shape() {
        return Err(PhysicsError::new(
            crate::PhysicsErrorEnum::DimensionMismatch(format!(
                "Innovation covariance shape {:?} != measurement noise shape {:?}",
                hph_t.shape(),
                measurement_noise.shape()
            )),
        ));
    }
    let s = hph_t.add(measurement_noise);

    // 3. Optimal Kalman Gain: K = P * H^T * S^-1
    // S^-1
    let s_inv = s.inverse().map_err(PhysicsError::from)?;

    // P * H^T
    let pht = p_pred.matmul(&ht).map_err(PhysicsError::from)?;

    // K = ... * S^-1
    let k = pht.matmul(&s_inv).map_err(PhysicsError::from)?;

    // 4. State Update: x_new = x + K * y
    // K * y
    let ky = k.matmul(&y).map_err(PhysicsError::from)?;

    if x_pred.shape() != ky.shape() {
        return Err(PhysicsError::new(
            crate::PhysicsErrorEnum::DimensionMismatch(format!(
                "State shape {:?} != update shape {:?}",
                x_pred.shape(),
                ky.shape()
            )),
        ));
    }
    let x_new = x_pred.add(&ky);

    // 5. Covariance Update (Joseph form):
    // P_new = (I - K H) P (I - K H)^T + K R K^T

    // K * H
    let kh = k.matmul(measurement_matrix).map_err(PhysicsError::from)?;

    // I (Identity matrix matching P dimension)
    let shape = p_pred.shape();
    let identity = CausalTensor::identity(shape).map_err(PhysicsError::from)?;

    // I - KH
    if identity.shape() != kh.shape() {
        return Err(PhysicsError::new(
            crate::PhysicsErrorEnum::DimensionMismatch(format!(
                "Identity shape {:?} != KH shape {:?}",
                identity.shape(),
                kh.shape()
            )),
        ));
    }
    let i_kh = identity.sub(&kh);

    // ... * P
    // (I - K H) P
    let left = i_kh.matmul(p_pred).map_err(PhysicsError::from)?;

    // (I - K H)^T
    let i_kh_t = {
        let op_t = EinSumOp::<f64>::transpose(i_kh.clone(), vec![1, 0]);
        CausalTensor::ein_sum(&op_t).map_err(PhysicsError::from)?
    };

    // (I - K H) P (I - K H)^T
    let joseph_main = left.matmul(&i_kh_t).map_err(PhysicsError::from)?;

    // K R K^T
    // K R K^T
    let kt = {
        let op_t = EinSumOp::<f64>::transpose(k.clone(), vec![1, 0]);
        CausalTensor::ein_sum(&op_t).map_err(PhysicsError::from)?
    };
    let kr = k.matmul(measurement_noise).map_err(PhysicsError::from)?;
    let krkt = kr.matmul(&kt).map_err(PhysicsError::from)?;

    let p_new = joseph_main.add(&krkt);

    // 6. Process Noise Addition: P_final = P_new + Q
    // We apply process noise here effectively preparing P for the next prediction step (or representing posterior uncertainty including process diffusion).
    if p_new.shape() != process_noise.shape() {
        return Err(PhysicsError::new(
            crate::PhysicsErrorEnum::DimensionMismatch(format!(
                "Posterior covariance shape {:?} != process noise shape {:?}",
                p_new.shape(),
                process_noise.shape()
            )),
        ));
    }
    let p_final = p_new.add(process_noise);

    Ok((x_new, p_final))
}
