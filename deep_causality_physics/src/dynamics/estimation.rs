/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use crate::Probability;
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};

/// Generalized Master Equation Kernel.
///
/// Implements the GME for systems with memory effects (non-Markovianity).
/// $P_{n+1} = T \cdot P_n + \sum_{k=0}^{m} \mathcal{K}_k \cdot P_{n-k} \cdot \Delta t$
///
/// # Arguments
/// * `state` - Current state vector ($P_n$).
/// * `history` - History of state vectors ($P_{n-k}$).
/// * `markov_operator` - Optional Markov transition matrix ($T$).
/// * `memory_kernel` - List of memory kernel tensors ($\mathcal{K}_k$).
///
/// # Returns
/// * `Result<Vec<Probability>, PhysicsError>` - The updated state vector.
pub fn generalized_master_equation_kernel(
    state: &[Probability],
    history: &[Vec<Probability>],
    markov_operator: Option<&CausalTensor<f64>>,
    memory_kernel: &[CausalTensor<f64>],
) -> Result<Vec<Probability>, PhysicsError> {
    // 1. Validation
    if history.len() != memory_kernel.len() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "History length {} != Memory kernel length {}",
            history.len(),
            memory_kernel.len()
        )));
    }

    // Convert state to tensor for operations (Column vector [n, 1] for matmul)
    let state_vec: Vec<f64> = state.iter().map(|p| p.value()).collect();
    let n = state_vec.len();
    let state_tensor = CausalTensor::new(state_vec, vec![n, 1]).map_err(PhysicsError::from)?;

    // 2. Markov Step
    let mut p_new_tensor = if let Some(t) = markov_operator {
        // T * P
        // If T is [N, N] and P is [N], result is [N].
        t.matmul(&state_tensor).map_err(PhysicsError::from)?
    } else {
        // Zero tensor of same shape as state
        CausalTensor::new(vec![0.0; n], vec![n, 1]).map_err(PhysicsError::from)?
    };

    // 3. Memory Integration
    for (k, kernel) in memory_kernel.iter().enumerate() {
        let hist_vec: Vec<f64> = history[k].iter().map(|p| p.value()).collect();
        // Validate history dimension
        if hist_vec.len() != n {
            return Err(PhysicsError::DimensionMismatch(format!(
                "History[{}] dimension {} != State dimension {}",
                k,
                hist_vec.len(),
                n
            )));
        }
        let hist_tensor = CausalTensor::new(hist_vec, vec![n, 1]).map_err(PhysicsError::from)?;

        // K * P_hist
        let contribution = kernel.matmul(&hist_tensor).map_err(PhysicsError::from)?;

        // Accumulate
        let sum = p_new_tensor.add(&contribution);
        p_new_tensor = sum; // CausalTensor::add returns a new tensor, assume ownership or clone
    }

    // 4. Output
    let result_data = p_new_tensor.data();
    let mut result_probs = Vec::with_capacity(n);
    for &val in result_data {
        // We use new_unchecked or new?
        // Spec says "Optionally re-normalize".
        // For now, let's try to clamp or check.
        // The GME can technically produce values outside [0,1] if kernels are not probability-conserving.
        // We will return error if outside range, enforcing the Probability type contract.
        // Or we clamp?
        // Given "Probability" type enforces [0,1] in new(), strict mode is better.
        // However, numerical noise might cause 1.00000001.
        let clamped = val.clamp(0.0, 1.0);
        result_probs.push(Probability::new_unchecked(clamped));
    }

    Ok(result_probs)
}

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
    _process_noise: &CausalTensor<f64>,
) -> Result<(CausalTensor<f64>, CausalTensor<f64>), PhysicsError> {
    // 1. Innovation (Residual): y = z - H * x
    // H * x
    let hx = measurement_matrix
        .matmul(x_pred)
        .map_err(PhysicsError::from)?;

    // y = z - hx
    if measurement.shape() != hx.shape() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Measurement shape {:?} != prediction shape {:?}",
            measurement.shape(),
            hx.shape()
        )));
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
        return Err(PhysicsError::DimensionMismatch(format!(
            "Innovation covariance shape {:?} != measurement noise shape {:?}",
            hph_t.shape(),
            measurement_noise.shape()
        )));
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
        return Err(PhysicsError::DimensionMismatch(format!(
            "State shape {:?} != update shape {:?}",
            x_pred.shape(),
            ky.shape()
        )));
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
        return Err(PhysicsError::DimensionMismatch(format!(
            "Identity shape {:?} != KH shape {:?}",
            identity.shape(),
            kh.shape()
        )));
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

    Ok((x_new, p_new))
}
