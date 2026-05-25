/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::kernels::fluids::{kinematics, mechanics};
use crate::{
    Density, Length, Pressure, RotationRateTensor, Speed, StrainRateTensor, Velocity3,
    VelocityGradient, VorticityVector,
};
use core::fmt::Debug;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_num::{FromPrimitive, RealField};

/// Causal wrapper for [`mechanics::hydrostatic_pressure_kernel`].
pub fn hydrostatic_pressure<R>(
    p0: &Pressure<R>,
    density: &Density<R>,
    depth: &Length<R>,
) -> PropagatingEffect<Pressure<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match mechanics::hydrostatic_pressure_kernel(p0, density, depth) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::bernoulli_pressure_kernel`].
pub fn bernoulli_pressure<R>(
    p1: &Pressure<R>,
    v1: &Speed<R>,
    h1: &Length<R>,
    v2: &Speed<R>,
    h2: &Length<R>,
    density: &Density<R>,
) -> PropagatingEffect<Pressure<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match mechanics::bernoulli_pressure_kernel(p1, v1, h1, v2, h2, density) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// =============================================================================
// Kinematic kernel wrappers
// =============================================================================

/// Causal wrapper for [`kinematics::strain_rate_tensor_kernel`].
pub fn strain_rate_tensor<R>(grad_u: &VelocityGradient<R>) -> PropagatingEffect<StrainRateTensor<R>>
where
    R: RealField + FromPrimitive + Debug + 'static,
{
    match kinematics::strain_rate_tensor_kernel(grad_u) {
        Ok(s) => PropagatingEffect::pure(s),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`kinematics::rotation_rate_tensor_kernel`].
pub fn rotation_rate_tensor<R>(
    grad_u: &VelocityGradient<R>,
) -> PropagatingEffect<RotationRateTensor<R>>
where
    R: RealField + FromPrimitive + Debug + 'static,
{
    match kinematics::rotation_rate_tensor_kernel(grad_u) {
        Ok(o) => PropagatingEffect::pure(o),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`kinematics::vorticity_from_gradient_kernel`].
pub fn vorticity_from_gradient<R>(
    grad_u: &VelocityGradient<R>,
) -> PropagatingEffect<VorticityVector<R>>
where
    R: RealField + Debug + 'static,
{
    PropagatingEffect::pure(kinematics::vorticity_from_gradient_kernel(grad_u))
}

/// Causal wrapper for [`kinematics::velocity_gradient_invariants_kernel`].
pub fn velocity_gradient_invariants<R>(grad_u: &VelocityGradient<R>) -> PropagatingEffect<(R, R, R)>
where
    R: RealField + FromPrimitive + Debug + Default + 'static,
{
    match kinematics::velocity_gradient_invariants_kernel(grad_u) {
        Ok(inv) => PropagatingEffect::pure(inv),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`kinematics::helicity_density_kernel`].
pub fn helicity_density<R>(u: &Velocity3<R>, omega: &VorticityVector<R>) -> PropagatingEffect<R>
where
    R: RealField + Debug + Default + 'static,
{
    PropagatingEffect::pure(kinematics::helicity_density_kernel(u, omega))
}

/// Causal wrapper for [`kinematics::enstrophy_density_kernel`].
pub fn enstrophy_density<R>(omega: &VorticityVector<R>) -> PropagatingEffect<R>
where
    R: RealField + FromPrimitive + Debug + Default + 'static,
{
    match kinematics::enstrophy_density_kernel(omega) {
        Ok(e) => PropagatingEffect::pure(e),
        Err(err) => PropagatingEffect::from_error(CausalityError::from(err)),
    }
}
