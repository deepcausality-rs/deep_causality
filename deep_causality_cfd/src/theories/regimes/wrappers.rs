/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Causal (`PropagatingEffect`) wrappers for the Navier–Stokes regime
//! evaluators — the theory layer migrated out of
//! `deep_causality_physics::kernels::fluids::wrappers`. Each lifts the regime
//! kernel's `Result` (or value) into the causal monad.

use super::{compressible_ns, euler, incompressible_ns, stokes};
use core::fmt::Debug;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_num::RealField;
use deep_causality_physics::{
    AccelerationVector, Density, KinematicViscosity, Velocity3, VelocityGradient,
};

/// Causal wrapper for [`incompressible_ns::incompressible_ns_rhs_kernel`].
pub fn incompressible_ns_rhs<R>(
    u: &Velocity3<R>,
    grad_u: &VelocityGradient<R>,
    laplacian_u: &[R; 3],
    grad_p: &[R; 3],
    rho: &Density<R>,
    nu: &KinematicViscosity<R>,
    body_force_per_mass: &AccelerationVector<R>,
) -> PropagatingEffect<AccelerationVector<R>>
where
    R: RealField + Debug + 'static,
{
    match incompressible_ns::incompressible_ns_rhs_kernel(
        u,
        grad_u,
        laplacian_u,
        grad_p,
        rho,
        nu,
        body_force_per_mass,
    ) {
        Ok(a) => PropagatingEffect::pure(a),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`euler::euler_momentum_rhs_kernel`].
pub fn euler_momentum_rhs<R>(
    u: &Velocity3<R>,
    grad_u: &VelocityGradient<R>,
    grad_p: &[R; 3],
    rho: &Density<R>,
    body_force_per_mass: &AccelerationVector<R>,
) -> PropagatingEffect<AccelerationVector<R>>
where
    R: RealField + Debug + 'static,
{
    match euler::euler_momentum_rhs_kernel(u, grad_u, grad_p, rho, body_force_per_mass) {
        Ok(a) => PropagatingEffect::pure(a),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`stokes::stokes_momentum_rhs_kernel`].
pub fn stokes_momentum_rhs<R>(
    laplacian_u: &[R; 3],
    grad_p: &[R; 3],
    rho: &Density<R>,
    nu: &KinematicViscosity<R>,
    body_force_per_mass: &AccelerationVector<R>,
) -> PropagatingEffect<AccelerationVector<R>>
where
    R: RealField + Debug + 'static,
{
    match stokes::stokes_momentum_rhs_kernel(laplacian_u, grad_p, rho, nu, body_force_per_mass) {
        Ok(a) => PropagatingEffect::pure(a),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`compressible_ns::compressible_ns_continuity_rhs_kernel`].
pub fn compressible_ns_continuity_rhs<R>(
    rho: &Density<R>,
    u: &Velocity3<R>,
    grad_rho: &[R; 3],
    div_u: R,
) -> PropagatingEffect<R>
where
    R: RealField + Debug + Default + 'static,
{
    PropagatingEffect::pure(compressible_ns::compressible_ns_continuity_rhs_kernel(
        rho, u, grad_rho, div_u,
    ))
}

/// Causal wrapper for [`compressible_ns::compressible_ns_momentum_rhs_kernel`].
pub fn compressible_ns_momentum_rhs<R>(
    u: &Velocity3<R>,
    grad_u: &VelocityGradient<R>,
    grad_p: &[R; 3],
    div_tau: &[R; 3],
    rho: &Density<R>,
    body_force_per_mass: &AccelerationVector<R>,
) -> PropagatingEffect<AccelerationVector<R>>
where
    R: RealField + Debug + 'static,
{
    match compressible_ns::compressible_ns_momentum_rhs_kernel(
        u,
        grad_u,
        grad_p,
        div_tau,
        rho,
        body_force_per_mass,
    ) {
        Ok(a) => PropagatingEffect::pure(a),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`compressible_ns::compressible_ns_energy_rhs_kernel`].
#[allow(clippy::too_many_arguments)]
pub fn compressible_ns_energy_rhs<R>(
    rho: &Density<R>,
    u: &Velocity3<R>,
    div_rho_u_e: R,
    div_p_u: R,
    div_tau_dot_u: R,
    div_q: R,
    body_force_per_mass: &AccelerationVector<R>,
) -> PropagatingEffect<R>
where
    R: RealField + Debug + Default + 'static,
{
    PropagatingEffect::pure(compressible_ns::compressible_ns_energy_rhs_kernel(
        rho,
        u,
        div_rho_u_e,
        div_p_u,
        div_tau_dot_u,
        div_q,
        body_force_per_mass,
    ))
}
