/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{PhysicsError, Temperature};
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};

/// Calculates generalized Hooke's Law: $\sigma_{ij} = C_{ijkl} \epsilon_{kl}$.
///
/// Computes stress tensor from stiffness tensor (Rank 4) and strain tensor (Rank 2) via Einstein Summation.
///
/// # Arguments
/// * `stiffness` - Stiffness tensor $C$ (Rank 4).
/// * `strain` - Strain tensor $\epsilon$ (Rank 2).
///
/// # Returns
/// * `Ok(CausalTensor<f64>)` - Stress tensor $\sigma$ (Rank 2).
pub fn hookes_law_kernel(
    stiffness: &CausalTensor<f64>,
    strain: &CausalTensor<f64>,
) -> Result<CausalTensor<f64>, PhysicsError> {
    // Sigma_ij = C_ijkl * Epsilon_kl
    // Stiffness C is Rank 4 [i, j, k, l]
    if stiffness.num_dim() != 4 || strain.num_dim() != 2 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Hooke's Law requires Stiffness Rank 4 and Strain Rank 2. Got {} and {}",
            stiffness.num_dim(),
            strain.num_dim()
        )));
    }

    let op = EinSumOp::<f64>::contraction(
        stiffness.clone(),
        strain.clone(),
        vec![2, 3], // C indices k, l
        vec![0, 1], // E indices k, l
    );

    // Execute EinSum
    let res = CausalTensor::ein_sum(&op)?;
    Ok(res)
}

/// Calculates Von Mises Stress from a 3x3 Stress Tensor.
///
/// $\sigma_{vm} = \sqrt{3 J_2}$ where $J_2 = \frac{1}{2} S_{ij}S_{ij}$ is the second invariant of deviatoric stress.
///
/// # Arguments
/// * `stress` - Cauchy stress tensor (3x3).
///
/// # Returns
/// * `Ok(Stress)` - Von Mises stress scalar.
pub fn von_mises_stress_kernel(stress: &CausalTensor<f64>) -> Result<crate::Stress, PhysicsError> {
    // Von Mises Stress via Deviatoric Stress Invariant J2
    // sigma_vm = sqrt(3 * J2)
    // J2 = 0.5 * (S : S)
    // S = sigma - sigma_m * I
    // sigma_m = tr(sigma) / 3

    if stress.num_dim() != 2 || stress.shape() != [3, 3] {
        return Err(PhysicsError::DimensionMismatch(
            "Von Mises requires 3x3 Stress Tensor".into(),
        ));
    }

    // 1. Calculate Mean Stress (Hydrostatic)
    // EinSumOp::trace(operand, axis1, axis2)
    // axes 0, 1
    let trace_op = EinSumOp::<f64>::trace(stress.clone(), 0, 1);
    let trace_tensor = CausalTensor::ein_sum(&trace_op)?; // Should be scalar (Rank 0) or 1 element?
    // Trace returns a tensor with reduced dimensions. For 2D, it should be 0D (Scalar).

    // Extract value
    let trace_val = if trace_tensor.shape().is_empty()
        || (trace_tensor.shape().len() == 1 && trace_tensor.shape()[0] == 1)
    {
        trace_tensor.data()[0]
    } else {
        return Err(PhysicsError::CalculationError("Trace failed".into()));
    };

    let sigma_m = trace_val / 3.0;

    // 2. Calculate Deviatoric Stress S
    // S = sigma - sigma_m * I
    let identity = CausalTensor::identity(&[3, 3])?;
    let mean_stress_tensor = identity * sigma_m;
    let s_deviatoric: CausalTensor<f64> = stress.clone() - mean_stress_tensor; // Assuming Sub impl works

    // 3. Calculate J2 = 0.5 * (S : S)
    // Double dot product / Full contraction
    // Contraction of S[0,1] with S[0,1] ?
    // Or element-wise separation then sum?
    // S_ij S_ij summation.
    // Use EinSum contraction on all axes.
    let j2_op = EinSumOp::<f64>::contraction(
        s_deviatoric.clone(),
        s_deviatoric.clone(),
        vec![0, 1],
        vec![0, 1],
    );
    let j2_tensor = CausalTensor::ein_sum(&j2_op)?;
    let j2_val = if j2_tensor.shape().is_empty()
        || (j2_tensor.shape().len() == 1 && j2_tensor.shape()[0] == 1)
    {
        j2_tensor.data()[0]
    } else {
        // Fallback for full contraction if it didn't reduce fully (should not happen with generic contraction logic)
        return Err(PhysicsError::CalculationError(
            "J2 calculation failed".into(),
        ));
    };

    let j2 = 0.5 * j2_val;

    // 4. Sigma_vm
    let vm = f64::sqrt(3.0 * j2);

    crate::Stress::new(vm)
}

/// Calculates thermal expansion strain: $\epsilon = \alpha \Delta T$.
///
/// # Arguments
/// * `coeff` - Thermal expansion coefficient $\alpha$.
/// * `delta_temp` - Change in temperature $\Delta T$.
///
/// # Returns
/// * `Ok(CausalTensor<f64>)` - Isotropic strain tensor (3x3).
pub fn thermal_expansion_kernel(
    coeff: f64,
    delta_temp: Temperature,
) -> Result<CausalTensor<f64>, PhysicsError> {
    // epsilon_ij = alpha * dT * delta_ij
    let val = coeff * delta_temp.value();
    let identity = CausalTensor::<f64>::identity(&[3, 3])?;
    let strain = identity * val;
    Ok(strain)
}
