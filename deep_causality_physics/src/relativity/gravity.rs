/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::PhysicsError;
use deep_causality_num::{Field, Float};
use deep_causality_tensor::CausalTensor;

// Kernels

/// Calculates the Einstein Tensor: $G_{\mu\nu} = R_{\mu\nu} - \frac{1}{2} R g_{\mu\nu}$.
///
/// # Arguments
/// * `ricci` - Ricci curvature tensor $R_{\mu\nu}$ (Rank 2).
/// * `scalar_r` - Ricci scalar $R$.
/// * `metric` - Metric tensor $g_{\mu\nu}$ (Rank 2).
///
/// # Returns
/// * `Ok(CausalTensor<T>)` - Einstein tensor $G_{\mu\nu}$.
pub fn einstein_tensor_kernel<T>(
    ricci: &CausalTensor<T>,
    scalar: T,
    metric: &CausalTensor<T>,
) -> Result<CausalTensor<T>, PhysicsError>
where
    T: Field + Float + From<f64>,
{
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

    // 0.5 * R
    let half_scalar = scalar * <T as From<f64>>::from(0.5);

    // (0.5 * R) * g_uv
    // Manual scalar multiplication since Mul<T> is not implemented for generic T
    let term2_data: Vec<T> = metric.as_slice().iter().map(|&x| x * half_scalar).collect();
    let term2 = CausalTensor::from_vec(term2_data, metric.shape());

    // R_uv - term2
    let result = ricci - term2;

    Ok(result)
}

/// Calculates Geodesic Deviation acceleration: $A^\mu = -R^\mu_{\nu\sigma\rho} V^\nu n^\sigma V^\rho$.
///
/// Computes contraction directly without einsum for generic T compatibility.
///
/// # Arguments
/// * `riemann` - Riemann curvature tensor $R^\mu_{\nu\sigma\rho}$ (Rank 4, shape [4,4,4,4]).
/// * `u` - Velocity vector $V^\nu$ (length 4).
/// * `n` - Separation vector $n^\sigma$ (length 4).
///
/// # Returns
/// * `Ok(Vec<T>)` - Relative acceleration vector $A^\mu$.
pub fn geodesic_deviation_kernel<T>(
    riemann: &CausalTensor<T>,
    u: &[T],
    n: &[T],
) -> Result<Vec<T>, PhysicsError>
where
    T: Field + Float + From<f64>,
{
    // A^u = - R^u_vrs U^v n^r U^s
    // Direct contraction without einsum for generic T

    let dim = 4usize;
    if riemann.num_dim() != 4 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Riemann tensor must be rank-4, got {}",
            riemann.num_dim()
        )));
    }
    if u.len() != dim || n.len() != dim {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Vectors must have length 4, got u={}, n={}",
            u.len(),
            n.len()
        )));
    }

    let r_data = riemann.as_slice();
    let mut result = vec![T::zero(); dim];

    // Contract: A^mu = -R^mu_vrs * U^v * n^r * U^s
    for (mu, res_mu) in result.iter_mut().enumerate() {
        let mut acc = T::zero();
        for v in 0..dim {
            for (r, n_r) in n.iter().enumerate() {
                for s in 0..dim {
                    // R^mu_vrs at index [mu, v, r, s]
                    let idx = mu * dim * dim * dim + v * dim * dim + r * dim + s;
                    let r_component = r_data[idx];
                    acc = acc + r_component * u[v] * *n_r * u[s];
                }
            }
        }
        *res_mu = T::zero() - acc; // Negate
    }

    Ok(result)
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
/// * `Ok(Vec<(Vec<T>, Vec<T>)>)` - List of (position, velocity) pairs along the geodesic.
#[allow(clippy::type_complexity)]
pub fn geodesic_integrator_kernel<T>(
    initial_position: &[T],
    initial_velocity: &[T],
    christoffel: &CausalTensor<T>,
    proper_time_step: T,
    num_steps: usize,
) -> Result<Vec<(Vec<T>, Vec<T>)>, PhysicsError>
where
    T: Field + Float + From<f64> + Copy,
{
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

    if !proper_time_step.is_finite() || proper_time_step == T::zero() {
        return Err(PhysicsError::NumericalInstability(
            "Invalid proper time step".into(),
        ));
    }

    let dt = proper_time_step;
    let christoffel_data = christoffel.as_slice();

    // Helper to compute acceleration: a^mu = -Gamma^mu_nu_rho * u^nu * u^rho
    let compute_acceleration = |u: &[T]| -> Vec<T> {
        let mut a = vec![T::zero(); dim];
        for mu in 0..dim {
            let mut acc = T::zero();
            for nu in 0..dim {
                for rho in 0..dim {
                    // Index: Gamma[mu, nu, rho] = christoffel_data[mu * dim * dim + nu * dim + rho]
                    let gamma = christoffel_data[mu * dim * dim + nu * dim + rho];
                    acc = acc - (gamma * u[nu] * u[rho]);
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
    let mut x: Vec<T> = initial_position.to_vec();
    let mut u: Vec<T> = initial_velocity.to_vec();

    trajectory.push((x.clone(), u.clone()));

    for _ in 0..num_steps {
        // k1
        let a1 = compute_acceleration(&u);
        let k1_x: Vec<T> = u.iter().map(|&v| v * dt).collect();
        let k1_u: Vec<T> = a1.iter().map(|&a| a * dt).collect();

        // k2
        let u_mid1: Vec<T> = u
            .iter()
            .zip(k1_u.iter())
            .map(|(v, k)| *v + <T as From<f64>>::from(0.5) * *k)
            .collect();
        let a2 = compute_acceleration(&u_mid1);
        let k2_x: Vec<T> = u_mid1.iter().map(|&v| v * dt).collect();
        let k2_u: Vec<T> = a2.iter().map(|&a| a * dt).collect();

        // k3
        let u_mid2: Vec<T> = u
            .iter()
            .zip(k2_u.iter())
            .map(|(v, k)| *v + <T as From<f64>>::from(0.5) * *k)
            .collect();
        let a3 = compute_acceleration(&u_mid2);
        let k3_x: Vec<T> = u_mid2.iter().map(|&v| v * dt).collect();
        let k3_u: Vec<T> = a3.iter().map(|&a| a * dt).collect();

        // k4
        let u_end: Vec<T> = u.iter().zip(k3_u.iter()).map(|(v, k)| *v + *k).collect();
        let a4 = compute_acceleration(&u_end);
        let k4_x: Vec<T> = u_end.iter().map(|&v| v * dt).collect();
        let k4_u: Vec<T> = a4.iter().map(|&a| a * dt).collect();

        // Update
        for i in 0..dim {
            x[i] = x[i]
                + (k1_x[i]
                    + <T as From<f64>>::from(2.0) * k2_x[i]
                    + <T as From<f64>>::from(2.0) * k3_x[i]
                    + k4_x[i])
                    / <T as From<f64>>::from(6.0);
            u[i] = u[i]
                + (k1_u[i]
                    + <T as From<f64>>::from(2.0) * k2_u[i]
                    + <T as From<f64>>::from(2.0) * k3_u[i]
                    + k4_u[i])
                    / <T as From<f64>>::from(6.0);
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
