/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::{PhysicsError, PhysicsErrorEnum};
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};

// Kernels

/// Calculates the Einstein Tensor: $G_{\mu\nu} = R_{\mu\nu} - \frac{1}{2} R g_{\mu\nu}$.
///
/// # Arguments
/// * `ricci` - Ricci curvature tensor $R_{\mu\nu}$ (Rank 2).
/// * `scalar_r` - Ricci scalar $R$.
/// * `metric` - Metric tensor $g_{\mu\nu}$ (Rank 2).
///
/// # Returns
/// * `Ok(CausalTensor<f64>)` - Einstein tensor $G_{\mu\nu}$.
pub fn einstein_tensor_kernel(
    ricci: &CausalTensor<f64>,
    scalar_r: f64,
    metric: &CausalTensor<f64>,
) -> Result<CausalTensor<f64>, PhysicsError> {
    // Validate ranks and shapes
    if ricci.num_dim() != 2 || metric.num_dim() != 2 {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            format!(
                "Einstein tensor requires rank-2 tensors. Got ranks: ricci={}, metric={}",
                ricci.num_dim(),
                metric.num_dim()
            ),
        )));
    }
    if ricci.shape() != metric.shape() {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            format!(
                "Ricci and metric shapes must match. Got {:?} vs {:?}",
                ricci.shape(),
                metric.shape()
            ),
        )));
    }
    if ricci.shape().len() == 2 && ricci.shape()[0] != ricci.shape()[1] {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            format!("Ricci tensor must be square. Got {:?}", ricci.shape()),
        )));
    }

    // G_uv = R_uv - 0.5 * R * g_uv
    let half_r = 0.5 * scalar_r;
    let term2 = metric.clone() * half_r;
    let g = ricci.clone() - term2;
    Ok(g)
}

/// Calculates Geodesic Deviation acceleration: $A^\mu = -R^\mu_{\nu\sigma\rho} V^\nu n^\sigma V^\rho$.
///
/// Describes the relative acceleration of nearby geodesics.
///
/// # Arguments
/// * `riemann` - Riemann curvature tensor $R^\mu_{\nu\sigma\rho}$ (Rank 4).
/// * `velocity` - Tangent vector $V^\nu$ (Rank 1).
/// * `separation` - Separation vector $n^\sigma$ (Rank 1).
///
/// # Returns
/// * `Ok(CausalTensor<f64>)` - Relative acceleration vector $A^\mu$.
pub fn geodesic_deviation_kernel(
    riemann: &CausalTensor<f64>,    // R^u_vsp (Rank 4)
    velocity: &CausalTensor<f64>,   // V^v (Rank 1)
    separation: &CausalTensor<f64>, // n^s (Rank 1)
) -> Result<CausalTensor<f64>, PhysicsError> {
    // Validate ranks
    if riemann.num_dim() != 4 || velocity.num_dim() != 1 || separation.num_dim() != 1 {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            format!(
                "Geodesic Deviation requires Riemann Rank 4, Velocity Rank 1, Separation Rank 1. Got {}, {}, {}",
                riemann.num_dim(),
                velocity.num_dim(),
                separation.num_dim()
            ),
        )));
    }

    // A^u = - R^u_vsp * V^v * n^s * V^p
    // Indices:
    // Riemann R[u, v, s, p] (0, 1, 2, 3)
    // V1[v] (0)
    // n[s] (0)
    // V2[p] (0)

    // Contraction sequence:
    // 1. R[u, v, s, p] * V[p] -> T1[u, v, s] (Contract R:3, V:0)
    // 2. T1[u, v, s] * n[s] -> T2[u, v] (Contract T1:2, n:0)
    // 3. T2[u, v] * V[v] -> A[u] (Contract T2:1, V:0)

    // 1. Contract Riemann with V (index p, last index of R)
    let op1 = EinSumOp::<f64>::contraction(riemann.clone(), velocity.clone(), vec![3], vec![0]);
    let t1 = CausalTensor::ein_sum(&op1)?;

    // 2. Contract T1 with n (index s, last index of T1)
    let op2 = EinSumOp::<f64>::contraction(t1, separation.clone(), vec![2], vec![0]);
    let t2 = CausalTensor::ein_sum(&op2)?;

    // 3. Contract T2 with V (index v, last index of T2)
    let op3 = EinSumOp::<f64>::contraction(t2, velocity.clone(), vec![1], vec![0]);
    let a_tensor = CausalTensor::ein_sum(&op3)?;

    // 4. Negate ( A = - ... )
    let a_neg = a_tensor * -1.0;

    Ok(a_neg)
}
