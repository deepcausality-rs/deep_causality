/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::kernels::fluids::{constitutive, governing, kinematics, mechanics};
use crate::{
    AccelerationVector, CauchyStress, Density, KinematicViscosity, Length, Pressure,
    RotationRateTensor, Speed, StrainRateTensor, Velocity3, VelocityGradient, Viscosity,
    VorticityVector,
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

// =============================================================================
// Governing-equation kernel wrappers
// =============================================================================

/// Causal wrapper for [`governing::convective_acceleration_kernel`].
pub fn convective_acceleration<R>(
    u: &Velocity3<R>,
    grad_u: &VelocityGradient<R>,
) -> PropagatingEffect<AccelerationVector<R>>
where
    R: RealField + Debug + 'static,
{
    PropagatingEffect::pure(governing::convective_acceleration_kernel(u, grad_u))
}

/// Causal wrapper for [`governing::viscous_diffusion_kernel`].
pub fn viscous_diffusion<R>(
    nu: &KinematicViscosity<R>,
    laplacian_u: &[R; 3],
) -> PropagatingEffect<AccelerationVector<R>>
where
    R: RealField + Debug + 'static,
{
    PropagatingEffect::pure(governing::viscous_diffusion_kernel(nu, laplacian_u))
}

/// Causal wrapper for [`governing::pressure_gradient_force_kernel`].
pub fn pressure_gradient_force<R>(
    rho: &Density<R>,
    grad_p: &[R; 3],
) -> PropagatingEffect<AccelerationVector<R>>
where
    R: RealField + Debug + 'static,
{
    match governing::pressure_gradient_force_kernel(rho, grad_p) {
        Ok(a) => PropagatingEffect::pure(a),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`governing::continuity_rhs_kernel`].
pub fn continuity_rhs<R>(
    rho: &Density<R>,
    u: &Velocity3<R>,
    grad_rho: &[R; 3],
    div_u: R,
) -> PropagatingEffect<R>
where
    R: RealField + Debug + Default + 'static,
{
    PropagatingEffect::pure(governing::continuity_rhs_kernel(rho, u, grad_rho, div_u))
}

/// Causal wrapper for [`governing::vorticity_transport_kernel`].
pub fn vorticity_transport<R>(
    omega: &VorticityVector<R>,
    u: &Velocity3<R>,
    grad_u: &VelocityGradient<R>,
    grad_omega: &[[R; 3]; 3],
    laplacian_omega: &[R; 3],
    nu: &KinematicViscosity<R>,
) -> PropagatingEffect<AccelerationVector<R>>
where
    R: RealField + Debug + 'static,
{
    PropagatingEffect::pure(governing::vorticity_transport_kernel(
        omega,
        u,
        grad_u,
        grad_omega,
        laplacian_omega,
        nu,
    ))
}

/// Causal wrapper for [`governing::scalar_advection_diffusion_kernel`].
pub fn scalar_advection_diffusion<R>(
    u: &Velocity3<R>,
    grad_phi: &[R; 3],
    laplacian_phi: R,
    diffusivity: R,
    source: R,
) -> PropagatingEffect<R>
where
    R: RealField + Debug + Default + 'static,
{
    PropagatingEffect::pure(governing::scalar_advection_diffusion_kernel(
        u,
        grad_phi,
        laplacian_phi,
        diffusivity,
        source,
    ))
}

/// Causal wrapper for [`governing::kinetic_energy_density_kernel`].
pub fn kinetic_energy_density<R>(rho: &Density<R>, u: &Velocity3<R>) -> PropagatingEffect<R>
where
    R: RealField + FromPrimitive + Debug + Default + 'static,
{
    match governing::kinetic_energy_density_kernel(rho, u) {
        Ok(e) => PropagatingEffect::pure(e),
        Err(err) => PropagatingEffect::from_error(CausalityError::from(err)),
    }
}

/// Causal wrapper for [`governing::viscous_dissipation_rate_kernel`].
pub fn viscous_dissipation_rate<R>(
    tau: &CauchyStress<R>,
    grad_u: &VelocityGradient<R>,
) -> PropagatingEffect<R>
where
    R: RealField + Debug + Default + 'static,
{
    PropagatingEffect::pure(governing::viscous_dissipation_rate_kernel(tau, grad_u))
}

/// Causal wrapper for [`governing::pressure_work_kernel`].
pub fn pressure_work<R>(p: &Pressure<R>, div_u: R) -> PropagatingEffect<R>
where
    R: RealField + Debug + Default + 'static,
{
    PropagatingEffect::pure(governing::pressure_work_kernel(p, div_u))
}

// =============================================================================
// Constitutive kernel wrappers
// =============================================================================

/// Causal wrapper for [`constitutive::newtonian_viscous_stress_kernel`].
pub fn newtonian_viscous_stress<R>(
    mu: &Viscosity<R>,
    strain_rate: &StrainRateTensor<R>,
    div_u: R,
) -> PropagatingEffect<CauchyStress<R>>
where
    R: RealField + FromPrimitive + Debug + 'static,
{
    match constitutive::newtonian_viscous_stress_kernel(mu, strain_rate, div_u) {
        Ok(tau) => PropagatingEffect::pure(tau),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`constitutive::newtonian_viscous_stress_with_bulk_kernel`].
pub fn newtonian_viscous_stress_with_bulk<R>(
    mu: &Viscosity<R>,
    zeta: &Viscosity<R>,
    strain_rate: &StrainRateTensor<R>,
    div_u: R,
) -> PropagatingEffect<CauchyStress<R>>
where
    R: RealField + FromPrimitive + Debug + 'static,
{
    match constitutive::newtonian_viscous_stress_with_bulk_kernel(mu, zeta, strain_rate, div_u) {
        Ok(tau) => PropagatingEffect::pure(tau),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`constitutive::power_law_apparent_viscosity_kernel`].
pub fn power_law_apparent_viscosity<R>(
    consistency: R,
    flow_index: R,
    shear_rate: R,
) -> PropagatingEffect<Viscosity<R>>
where
    R: RealField + Debug + 'static,
{
    match constitutive::power_law_apparent_viscosity_kernel(consistency, flow_index, shear_rate) {
        Ok(mu) => PropagatingEffect::pure(mu),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
