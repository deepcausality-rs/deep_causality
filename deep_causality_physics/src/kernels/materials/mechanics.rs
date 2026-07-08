/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{PhysicsError, StiffnessTensor, Strain, Stress, StressTensor, Temperature};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
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
/// * `Ok(StressTensor<R>)` - Stress tensor $\sigma$ (Rank 2).
pub fn hookes_law_kernel<R>(
    stiffness: &StiffnessTensor<R>,
    strain: &Strain<R>,
) -> Result<StressTensor<R>, PhysicsError>
where
    R: RealField + Default,
{
    if stiffness.inner().num_dim() != 4 || strain.inner().num_dim() != 2 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Hooke's Law requires Stiffness Rank 4 and Strain Rank 2. Got {} and {}",
            stiffness.inner().num_dim(),
            strain.inner().num_dim()
        )));
    }

    let op = EinSumOp::<R>::contraction(
        stiffness.inner().clone(),
        strain.inner().clone(),
        vec![2, 3], // C indices k, l
        vec![0, 1], // E indices k, l
    );

    let res = CausalTensor::ein_sum(&op)?;
    Ok(StressTensor::new(res))
}

/// Calculates Von Mises Stress from a 3x3 Stress Tensor.
///
/// $\sigma_{vm} = \sqrt{3 J_2}$ where $J_2 = \frac{1}{2} S_{ij}S_{ij}$ is the second invariant of deviatoric stress.
///
/// # Arguments
/// * `stress` - Cauchy stress tensor (3x3).
///
/// # Returns
/// * `Ok(Stress<R>)` - Von Mises stress scalar.
pub fn von_mises_stress_kernel<R>(stress: &StressTensor<R>) -> Result<Stress<R>, PhysicsError>
where
    R: RealField + Default + FromPrimitive,
{
    let s = stress.inner();
    if s.num_dim() != 2 || s.shape() != [3, 3] {
        return Err(PhysicsError::DimensionMismatch(
            "Von Mises requires 3x3 Stress Tensor".into(),
        ));
    }

    // 1. Mean (hydrostatic) stress sigma_m = tr(sigma) / 3
    let trace_op = EinSumOp::<R>::trace(s.clone(), 0, 1);
    let trace_tensor = CausalTensor::ein_sum(&trace_op)?;
    let trace_val = if trace_tensor.shape().is_empty()
        || (trace_tensor.shape().len() == 1 && trace_tensor.shape()[0] == 1)
    {
        trace_tensor.data()[0]
    } else {
        return Err(PhysicsError::CalculationError("Trace failed".into()));
    };

    let three = R::from_f64(3.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(3.0) failed".into()))?;
    let sigma_m = trace_val / three;

    // 2. Deviatoric stress S = sigma - sigma_m * I
    let identity = CausalTensor::<R>::identity(&[3, 3])?;
    let mean_stress_tensor = identity * sigma_m;
    let s_deviatoric: CausalTensor<R> = s.clone() - mean_stress_tensor;

    // 3. J2 = 0.5 * (S : S)
    let j2_op = EinSumOp::<R>::contraction(
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
        return Err(PhysicsError::CalculationError(
            "J2 calculation failed".into(),
        ));
    };

    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let j2 = half * j2_val;

    // 4. Sigma_vm = sqrt(3 * J2)
    let vm = (three * j2).sqrt();
    Stress::new(vm)
}

/// Calculates thermal expansion strain: $\epsilon = \alpha \Delta T$.
///
/// # Arguments
/// * `coeff` - Thermal expansion coefficient $\alpha$.
/// * `delta_temp` - Change in temperature $\Delta T$.
///
/// # Returns
/// * `Ok(CausalTensor<R>)` - Isotropic strain tensor (3x3).
pub fn thermal_expansion_kernel<R>(
    coeff: R,
    delta_temp: Temperature<R>,
) -> Result<CausalTensor<R>, PhysicsError>
where
    R: deep_causality_algebra::RealField + Default + PartialOrd,
{
    // epsilon_ij = alpha * dT * delta_ij
    let val = coeff * delta_temp.value();
    let identity = CausalTensor::<R>::identity(&[3, 3])?;
    let strain = identity * val;
    Ok(strain)
}
