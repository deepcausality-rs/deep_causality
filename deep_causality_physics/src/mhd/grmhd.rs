/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{LorentzianMetric, PhysicsError};
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};
use deep_causality_topology::Manifold;

/// Calculates relativistic current density J^μ via covariant divergence.
///
/// # Physical Model
///
/// Computes the source current from Maxwell's equations:
/// $$ J^\mu = \nabla_\nu F^{\mu\nu} $$
///
/// Using differential forms on a simplicial complex:
/// $$ J = \delta F = \star d \star F $$
///
/// where δ is the codifferential operator.
///
/// # Sign Convention
///
/// Uses the `LorentzianMetric` trait to ensure consistent sign conventions.
/// Default is East Coast (-+++) via `PhysicsMetric`.
///
/// # Arguments
/// * `em_manifold` - Manifold with electromagnetic 2-form F data on 2-simplices
/// * `spacetime_metric` - Spacetime signature implementing `LorentzianMetric`
///
/// # Returns
/// Current density 1-form J as a `CausalTensor`
///
/// # Errors
/// - `DimensionMismatch`: Manifold dimension < 4 or metric dimension mismatch
/// - `CalculationError`: Missing differential operators
pub fn relativistic_current_kernel<M: LorentzianMetric>(
    em_manifold: &Manifold<f64, f64>,
    spacetime_metric: &M,
) -> Result<CausalTensor<f64>, PhysicsError> {
    let complex = em_manifold.complex();
    let skeletons = complex.skeletons();

    // 1. Validate dimensions
    if skeletons.len() < 3 {
        return Err(PhysicsError::DimensionMismatch(
            "Requires at least 2-simplices for EM 2-form".into(),
        ));
    }

    if spacetime_metric.dimension() < 4 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Spacetime needs 4D, got {}D",
            spacetime_metric.dimension()
        )));
    }

    // 2. Get operators from complex
    let hodge_ops = complex.hodge_star_operators();
    let coboundary_ops = complex.coboundary_operators();

    // Validate operators exist
    if hodge_ops.len() < 4 {
        return Err(PhysicsError::CalculationError(format!(
            "Missing Hodge star operators: need 4, have {}",
            hodge_ops.len()
        )));
    }

    if coboundary_ops.len() < 3 {
        return Err(PhysicsError::CalculationError(format!(
            "Missing coboundary operators: need 3, have {}",
            coboundary_ops.len()
        )));
    }

    // 3. Extract F as 2-form data from manifold
    // Data layout: [0-simplices | 1-simplices | 2-simplices | ...]
    let n0 = skeletons[0].simplices().len();
    let n1 = skeletons[1].simplices().len();
    let n2 = skeletons[2].simplices().len();

    let data = em_manifold.data().as_slice();
    if data.len() < n0 + n1 + n2 {
        return Err(PhysicsError::CalculationError(
            "Manifold data too short for 2-form extraction".into(),
        ));
    }

    let f_2form: Vec<f64> = data[n0 + n1..n0 + n1 + n2].to_vec();

    // 4. Compute J = ★d★F (codifferential of F)
    // Step 4a: ★F (apply Hodge star to 2-form)
    let star_f = apply_csr_f64(&hodge_ops[2], &f_2form);

    // Step 4b: d(★F) (apply coboundary/exterior derivative)
    let d_star_f = apply_csr_i8_f64(&coboundary_ops[2], &star_f);

    // Step 4c: ★(d★F) (apply Hodge star to get 1-form)
    let j_data = apply_csr_f64(&hodge_ops[3], &d_star_f);

    // 5. Return as CausalTensor (1-form)
    CausalTensor::new(j_data.clone(), vec![j_data.len()]).map_err(PhysicsError::from)
}

/// Helper: Apply CSR matrix with f64 values to f64 vector.
#[allow(clippy::needless_range_loop)]
fn apply_csr_f64(matrix: &CsrMatrix<f64>, vec: &[f64]) -> Vec<f64> {
    let n_rows = matrix.shape().0;
    let mut result = vec![0.0; n_rows];

    for row in 0..n_rows {
        let row_start = matrix.row_indices()[row];
        let row_end = matrix.row_indices()[row + 1];

        for idx in row_start..row_end {
            let col = matrix.col_indices()[idx];
            let val = matrix.values()[idx];
            if col < vec.len() {
                result[row] += val * vec[col];
            }
        }
    }

    result
}

/// Helper: Apply CSR matrix with i8 values to f64 vector.
#[allow(clippy::needless_range_loop)]
fn apply_csr_i8_f64(matrix: &CsrMatrix<i8>, vec: &[f64]) -> Vec<f64> {
    let n_rows = matrix.shape().0;
    let mut result = vec![0.0; n_rows];

    for row in 0..n_rows {
        let row_start = matrix.row_indices()[row];
        let row_end = matrix.row_indices()[row + 1];

        for idx in row_start..row_end {
            let col = matrix.col_indices()[idx];
            let val = matrix.values()[idx] as f64;
            if col < vec.len() {
                result[row] += val * vec[col];
            }
        }
    }

    result
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
        return Err(PhysicsError::DimensionMismatch(
            "Tensors must be Rank 2".into(),
        ));
    }

    // 1. Compute covariant F_αβ = g_αμ * F^μν * g_νβ
    // In matrix notation, this is F_lower = g * F * g
    let gf = metric.matmul(em_tensor)?;
    let f_lower = gf.matmul(metric)?;

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
        return Err(PhysicsError::CalculationError(
            "Scalar contraction failed".into(),
        ));
    };

    // 3. Compute Term 1: T1^uv = F^ua * F^v_a
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
