/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::PhysicsError;
use deep_causality_tensor::{CausalTensor, EinSumOp};

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
        return Err(PhysicsError::DimensionMismatch(format!(
            "Einstein tensor requires rank-2 tensors. Got ranks: ricci={}, metric={}",
            ricci.num_dim(),
            metric.num_dim()
        )));
    }
    if ricci.shape() != metric.shape() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Ricci and metric shapes must match. Got {:?} vs {:?}",
            ricci.shape(),
            metric.shape()
        )));
    }
    if ricci.shape().len() == 2 && ricci.shape()[0] != ricci.shape()[1] {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Ricci tensor must be square. Got {:?}",
            ricci.shape()
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
        return Err(PhysicsError::DimensionMismatch(format!(
            "Geodesic Deviation requires Riemann Rank 4, Velocity Rank 1, Separation Rank 1. Got {}, {}, {}",
            riemann.num_dim(),
            velocity.num_dim(),
            separation.num_dim()
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

/// Integrates the geodesic equation using 4th-order Runge-Kutta (RK4).
///
/// Solves: $\frac{d^2 x^\mu}{d\tau^2} + \Gamma^\mu_{\nu\rho} \frac{dx^\nu}{d\tau} \frac{dx^\rho}{d\tau} = 0$
///
/// # Arguments
/// * `initial_position` - Initial position $x_0^\mu$ (4-vector).
/// * `initial_velocity` - Initial 4-velocity $u_0^\mu = dx^\mu/d\tau$.
/// * `christoffel` - Christoffel symbols $\Gamma^\mu_{\nu\rho}$ (Rank 3 tensor [dim, dim, dim]).
/// * `proper_time_step` - Step size $d\tau$ in proper time.
/// * `num_steps` - Number of integration steps.
///
/// # Returns
/// * `Ok(Vec<(Vec<f64>, Vec<f64>)>)` - List of (position, velocity) pairs along the geodesic.
#[allow(clippy::type_complexity)]
pub fn geodesic_integrator_kernel(
    initial_position: &[f64],
    initial_velocity: &[f64],
    christoffel: &CausalTensor<f64>,
    proper_time_step: f64,
    num_steps: usize,
) -> Result<Vec<(Vec<f64>, Vec<f64>)>, PhysicsError> {
    // Validate inputs
    if initial_position.len() != initial_velocity.len() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Position and velocity must have same dimension: {} vs {}",
            initial_position.len(),
            initial_velocity.len()
        )));
    }

    let dim = initial_position.len();
    if christoffel.num_dim() != 3 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Christoffel symbols must be rank 3, got {}",
            christoffel.num_dim()
        )));
    }

    let shape = christoffel.shape();
    if shape[0] != dim || shape[1] != dim || shape[2] != dim {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Christoffel shape {:?} incompatible with dimension {}",
            shape, dim
        )));
    }

    if !proper_time_step.is_finite() || proper_time_step == 0.0 {
        return Err(PhysicsError::NumericalInstability(
            "Invalid proper time step".into(),
        ));
    }

    let dt = proper_time_step;
    let christoffel_data = christoffel.as_slice();

    // Helper to compute acceleration: a^mu = -Gamma^mu_nu_rho * u^nu * u^rho
    let compute_acceleration = |u: &[f64]| -> Vec<f64> {
        let mut a = vec![0.0; dim];
        for mu in 0..dim {
            let mut acc = 0.0;
            for nu in 0..dim {
                for rho in 0..dim {
                    // Index: Gamma[mu, nu, rho] = christoffel_data[mu * dim * dim + nu * dim + rho]
                    let gamma = christoffel_data[mu * dim * dim + nu * dim + rho];
                    acc -= gamma * u[nu] * u[rho];
                }
            }
            a[mu] = acc;
        }
        a
    };

    // RK4 for second-order ODE split into first-order system:
    // dx/dt = u
    // du/dt = a(u)
    let mut trajectory = Vec::with_capacity(num_steps + 1);
    let mut x: Vec<f64> = initial_position.to_vec();
    let mut u: Vec<f64> = initial_velocity.to_vec();

    trajectory.push((x.clone(), u.clone()));

    for _ in 0..num_steps {
        // k1
        let a1 = compute_acceleration(&u);
        let k1_x: Vec<f64> = u.iter().map(|&v| v * dt).collect();
        let k1_u: Vec<f64> = a1.iter().map(|&a| a * dt).collect();

        // k2
        let u_mid1: Vec<f64> = u
            .iter()
            .zip(k1_u.iter())
            .map(|(v, k)| v + 0.5 * k)
            .collect();
        let a2 = compute_acceleration(&u_mid1);
        let k2_x: Vec<f64> = u_mid1.iter().map(|&v| v * dt).collect();
        let k2_u: Vec<f64> = a2.iter().map(|&a| a * dt).collect();

        // k3
        let u_mid2: Vec<f64> = u
            .iter()
            .zip(k2_u.iter())
            .map(|(v, k)| v + 0.5 * k)
            .collect();
        let a3 = compute_acceleration(&u_mid2);
        let k3_x: Vec<f64> = u_mid2.iter().map(|&v| v * dt).collect();
        let k3_u: Vec<f64> = a3.iter().map(|&a| a * dt).collect();

        // k4
        let u_end: Vec<f64> = u.iter().zip(k3_u.iter()).map(|(v, k)| v + k).collect();
        let a4 = compute_acceleration(&u_end);
        let k4_x: Vec<f64> = u_end.iter().map(|&v| v * dt).collect();
        let k4_u: Vec<f64> = a4.iter().map(|&a| a * dt).collect();

        // Update
        for i in 0..dim {
            x[i] += (k1_x[i] + 2.0 * k2_x[i] + 2.0 * k3_x[i] + k4_x[i]) / 6.0;
            u[i] += (k1_u[i] + 2.0 * k2_u[i] + 2.0 * k3_u[i] + k4_u[i]) / 6.0;
        }

        // Check for numerical instability
        if x.iter().any(|v| !v.is_finite()) || u.iter().any(|v| !v.is_finite()) {
            return Err(PhysicsError::NumericalInstability(format!(
                "Geodesic integration diverged at step {}",
                trajectory.len()
            )));
        }

        trajectory.push((x.clone(), u.clone()));
    }

    Ok(trajectory)
}
