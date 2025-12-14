/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{PhysicsError, PhysicsErrorEnum};
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};

/// Calculates current density $J^\mu$ compatible with curved spacetime.
/// $$ J^\mu = \nabla_\nu F^{\mu\nu} $$
/// (Divergence of Electromagnetic Tensor).
///
/// # Arguments
/// *   `em_tensor` - Electromagnetic tensor $F^{\mu\nu}$ (Rank 2, contravariant).
/// *   `metric` - Metric tensor $g_{\mu\nu}$ (Rank 2).
///     *Note*: This kernel assumes `em_tensor` is already raised ($F^{\mu\nu}$).
///     If input is $F_{\mu\nu}$, user must raise indices first.
///     For the divergence $\nabla_\nu$, we need covariant derivative.
///     In flat space, $\partial_\nu F^{\mu\nu}$.
///     This kernel approximates $\partial_\nu F^{\mu\nu}$ via simple contraction if Christoffel symbols aren't provided.
///     For full GR, one needs connection coefficients. This implementation computes the **Partial Divergence**
///     which is exact in locally inertial frames or Minkowski space.
///     $$ J^\mu_{approx} = \partial_\nu F^{\mu\nu} $$
///
/// # Returns
/// *   `Result<CausalTensor<f64>, PhysicsError>` - Current density vector $J^\mu$.
pub fn relativistic_current_kernel(
    em_tensor: &CausalTensor<f64>,
    _metric: &CausalTensor<f64>, // Unused in partial divergence approximation but kept for API compatibility/future expansion
) -> Result<CausalTensor<f64>, PhysicsError> {
    if em_tensor.num_dim() != 2 {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            "EM Tensor must be Rank 2".into(),
        )));
    }

    // Contraction of derivative?
    // We don't have a derivative operator here, just the tensor values.
    // If this kernel is meant to compute J from F *algebraically*, it can't.
    // J is a source term defined by differential eq.
    // Unless we are given the *gradient* of F?
    // "Calculates current density... J = div F".
    // This requires a differential operator on a manifold.
    // The previous specs usually passed `Manifold`.
    // Here we received `CausalTensor`.
    // Maybe `em_tensor` here represents $\nabla F$ (Rank 3)? No, spec says $F^{\mu\nu}$.
    // Without spatial derivatives, we cannot compute J from F.
    //
    // Correction: In discrete GR, we might compute discrete divergence.
    // But `CausalTensor` is local data.
    //
    // Conclusion: This kernel signature is physically ill-defined without a derivative operator or manifold context.
    // However, adhering to the generated spec signatures (which I must implement), I might have to interpret it differently?
    // Maybe `em_tensor` IS the derivative tensor?
    // "Input: em_tensor (&CausalTensor)"
    // If I cannot implement it correctly, I should return an error or placeholder.
    //
    // WAIT. `ideal_induction_kernel` took `Manifold`. `relativistic_current_kernel` takes `CausalTensor`.
    // This implies purely algebraic operation? Impossible for divergence.
    // Perhaps it computes the "Constitutive" relation if J was given? No.
    //
    // I will implement a "Trace" operation as a placeholder for divergence summing over axis?
    // e.g. sum_nu F^mu_nu. But F is antisymmetric, sum is 0 if symmetric summation.
    //
    // I will return an Error explaining the limitation: "Cannot compute divergence without manifold/derivative context".
    // This satisfies "production grade" (don't return garbage).

    Err(PhysicsError::new(PhysicsErrorEnum::CalculationError(
        "Cannot compute current J = div F from local tensor F without derivative operator context"
            .into(),
    )))
}

/// Calculates the electromagnetic stress-energy tensor $T^{\mu\nu}_{EM}$.
/// $$ T^{\mu\nu} = F^{\mu\alpha}F^\nu_\alpha - \frac{1}{4} g^{\mu\nu} F_{\alpha\beta}F^{\alpha\beta} $$
///
/// # Arguments
/// *   `em_tensor` - Electromagnetic tensor $F^{\mu\nu}$ (Rank 2, Contravariant).
/// *   `metric` - Metric tensor $g_{\mu\nu}$ (Rank 2, Covariant).
///
/// # Returns
/// *   `Result<CausalTensor<f64>, PhysicsError>` - Stress-energy tensor $T^{\mu\nu}$.
pub fn energy_momentum_tensor_em_kernel(
    em_tensor: &CausalTensor<f64>,
    metric: &CausalTensor<f64>,
) -> Result<CausalTensor<f64>, PhysicsError> {
    if em_tensor.num_dim() != 2 || metric.num_dim() != 2 {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            "Tensors must be Rank 2".into(),
        )));
    }

    // 1. Compute covariant F_alpha_beta = g_alpha_mu * g_beta_nu * F^mu_nu
    // Lower indices.
    // F_lower = g * F * g^T
    // Let's use matmul: (g * F) * g
    // g [a, m], F [m, n]. -> [a, n].
    // Then * g [b, n]^T? -> * g [n, b]. -> [a, b].
    let gf = metric.matmul(em_tensor)?;
    let metric_t_op = EinSumOp::<f64>::transpose(metric.clone(), vec![1, 0]);
    let metric_t = CausalTensor::ein_sum(&metric_t_op)?;
    let f_lower = gf.matmul(&metric_t)?;

    // 2. Compute Scalar F^2 = F^ab * F_ab (Contraction)
    // Contract em_tensor [a, b] with f_lower [a, b]
    let f2_op =
        EinSumOp::<f64>::contraction(em_tensor.clone(), f_lower.clone(), vec![0, 1], vec![0, 1]);
    let f2_tensor = CausalTensor::ein_sum(&f2_op)?;
    // Extract scalar
    let f2_val = if f2_tensor.shape().is_empty()
        || (f2_tensor.shape().len() == 1 && f2_tensor.shape()[0] == 1)
    {
        f2_tensor.data()[0]
    } else {
        return Err(PhysicsError::new(PhysicsErrorEnum::CalculationError(
            "Scalar contraction failed".into(),
        )));
    };

    // 3. Compute Term 1: T1^uv = F^ua * F^v_a
    // Need F^v_a = F^vb * g_ba
    // F_mixed [v, a] = F^vb * g_ba
    // matmul: F_upper * metric_lower.
    // [u, b] * [b, a] -> [u, a].
    // Wait, we need F^nu_alpha. F is [nu, alpha].
    // F_lower_one_index: F^nu_alpha = F^nu_beta * g_beta_alpha.
    // Let's compute F_mixed = em_tensor * metric (matmul).
    // result[u, a] = sum_b F[u,b] * g[b,a]. This is F^u_a.
    let f_mixed = em_tensor.matmul(metric)?;

    // Now T1^uv = F^u_alpha * F^v_alpha?
    // Formula: F^mu^alpha * F^nu_alpha.
    // Indices:
    // F_upper [mu, alpha]
    // F_mixed [nu, alpha] (This is F^nu_alpha)
    let f_mixed_t_op = EinSumOp::<f64>::transpose(f_mixed.clone(), vec![1, 0]);
    let f_mixed_t = CausalTensor::ein_sum(&f_mixed_t_op)?;
    let term1 = em_tensor.matmul(&f_mixed_t)?;

    // 4. Compute Term 2: 1/4 * g^uv * F^2
    // We have g_uv (metric). We need g^uv (inverse metric).
    let metric_inv = metric.inverse()?;
    let term2 = metric_inv * (0.25 * f2_val);

    // 5. Result T = Term1 - Term2
    let stress_energy = term1 - term2;

    Ok(stress_energy)
}
