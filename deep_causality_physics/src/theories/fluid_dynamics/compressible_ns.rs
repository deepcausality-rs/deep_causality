/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Compressible Newtonian Navier-Stokes regime evaluators.
//!
//! Three pointwise RHS kernels for the compressible NS system:
//!
//! ```text
//! ∂ρ/∂t   = − ∇·(ρ u)
//! ∂u/∂t   = − (u·∇)u − (1/ρ) ∇p + (1/ρ) ∇·τ + g
//! ∂(ρE)/∂t = − ∇·(ρ u E) − ∇·(p u) + ∇·(τ·u) − ∇·q + ρ (u·g)
//! ```
//!
//! Conserved variables: `ρ` (density), `ρu` (momentum), `ρE` (total energy
//! per unit volume), with `E = e + 0.5‖u‖²`. The momentum kernel returns the
//! Eulerian acceleration `∂u/∂t` (primitive form) for consistency with the
//! other regime evaluators; the energy kernel returns the conservative-form
//! scalar `∂(ρE)/∂t`.
//!
//! Sign convention follows continuum mechanics: viscous stress positive in
//! tension; heat-flux vector `q` follows Fourier's law `q = −κ∇T`, so the
//! `-∇·q` term in the energy equation is a heat *source*.
//!
//! Caller computes the spatial divergences (`∇·τ`, `∇·q`, `∇·(p u)`,
//! `∇·(τ·u)`, `∇·(ρ u E)`) at the sample point; these kernels do not
//! discretise space.

use crate::PhysicsError;
use crate::kernels::fluids::governing::{
    continuity_rhs_kernel, convective_acceleration_kernel, pressure_gradient_force_kernel,
};
use crate::kernels::fluids::quantities::{
    AccelerationVector, Density, Velocity3, VelocityGradient,
};
use deep_causality_num::RealField;

/// Continuity equation RHS: `∂ρ/∂t = − u·∇ρ − ρ ∇·u`.
///
/// Reduces to `0` for incompressible divergence-free flow. Returned as a
/// scalar in kg/(m³·s).
pub fn compressible_ns_continuity_rhs_kernel<R>(
    rho: &Density<R>,
    u: &Velocity3<R>,
    grad_rho: &[R; 3],
    div_u: R,
) -> R
where
    R: RealField,
{
    continuity_rhs_kernel(rho, u, grad_rho, div_u)
}

/// Momentum equation RHS in primitive velocity form:
/// `∂u/∂t = − (u·∇)u − (1/ρ) ∇p + (1/ρ) ∇·τ + g`.
///
/// Reduces to `incompressible_ns_rhs_kernel` when the viscous-stress
/// divergence equals `ρ ν ∇²u` (constant `μ`, divergence-free flow).
///
/// - `div_tau`      — divergence of the viscous stress tensor (Pa/m)
/// - errors when `ρ = 0` (inherited from `pressure_gradient_force_kernel`)
pub fn compressible_ns_momentum_rhs_kernel<R>(
    u: &Velocity3<R>,
    grad_u: &VelocityGradient<R>,
    grad_p: &[R; 3],
    div_tau: &[R; 3],
    rho: &Density<R>,
    body_force_per_mass: &AccelerationVector<R>,
) -> Result<AccelerationVector<R>, PhysicsError>
where
    R: RealField,
{
    let r = rho.value();
    if r == R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "compressible_ns_momentum_rhs_kernel: density is zero".into(),
        ));
    }
    let inv_rho = R::one() / r;
    let conv = convective_acceleration_kernel(u, grad_u).into_inner();
    let press = pressure_gradient_force_kernel(rho, grad_p)?.into_inner();
    let g = body_force_per_mass.value();

    Ok(AccelerationVector::new_unchecked([
        -conv[0] + press[0] + inv_rho * div_tau[0] + g[0],
        -conv[1] + press[1] + inv_rho * div_tau[1] + g[1],
        -conv[2] + press[2] + inv_rho * div_tau[2] + g[2],
    ]))
}

/// Total-energy equation RHS in conservative form:
///
/// `∂(ρE)/∂t = − ∇·(ρ u E) − ∇·(p u) + ∇·(τ·u) − ∇·q + ρ (u·g)`
///
/// All four divergences are supplied by the caller at the sample point.
/// Sign of the heat-flux term follows the convention that `q` points along
/// `−∇T` (Fourier), so `−∇·q > 0` corresponds to net heat *deposited* at
/// the point.
pub fn compressible_ns_energy_rhs_kernel<R>(
    rho: &Density<R>,
    u: &Velocity3<R>,
    div_rho_u_e: R,
    div_p_u: R,
    div_tau_dot_u: R,
    div_q: R,
    body_force_per_mass: &AccelerationVector<R>,
) -> R
where
    R: RealField,
{
    let u_raw = u.value();
    let g = body_force_per_mass.value();
    let u_dot_g = u_raw[0] * g[0] + u_raw[1] * g[1] + u_raw[2] * g[2];
    -div_rho_u_e - div_p_u + div_tau_dot_u - div_q + rho.value() * u_dot_g
}
