/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stokes regime evaluator (creeping-flow limit of Navier-Stokes).
//!
//! Pointwise RHS of the linearised Stokes momentum equation:
//!
//! ```text
//! ∂u/∂t = − (1/ρ) ∇p + ν ∇²u + g
//! ```
//!
//! The nonlinear convective term `(u·∇)u` is dropped under the creeping-flow
//! assumption `Re → 0`. The signature therefore takes neither `u` nor
//! `grad_u`; the type encodes the limit. Errors propagate from
//! [`pressure_gradient_force_kernel`] when `ρ = 0`.

use deep_causality_num::RealField;
use deep_causality_physics::PhysicsError;
use deep_causality_physics::{AccelerationVector, Density, KinematicViscosity};
use deep_causality_physics::{pressure_gradient_force_kernel, viscous_diffusion_kernel};

/// Pointwise RHS of the Stokes momentum equation (creeping flow).
///
/// `∂u/∂t = − (1/ρ) ∇p + ν ∇²u + g`
///
/// - `laplacian_u`  — `∇²u` (1/(m·s))
/// - `grad_p`       — pressure gradient (Pa/m)
/// - `rho`          — fluid density (kg/m³); errors when zero
/// - `nu`           — kinematic viscosity (m²/s)
/// - `body_force_per_mass` — body acceleration (m/s²)
pub fn stokes_momentum_rhs<R>(
    laplacian_u: &[R; 3],
    grad_p: &[R; 3],
    rho: &Density<R>,
    nu: &KinematicViscosity<R>,
    body_force_per_mass: &AccelerationVector<R>,
) -> Result<AccelerationVector<R>, PhysicsError>
where
    R: RealField,
{
    let press = pressure_gradient_force_kernel(rho, grad_p)?.into_inner();
    let visc = viscous_diffusion_kernel(nu, laplacian_u).into_inner();
    let g = body_force_per_mass.value();

    Ok(AccelerationVector::new_unchecked([
        press[0] + visc[0] + g[0],
        press[1] + visc[1] + g[1],
        press[2] + visc[2] + g[2],
    ]))
}
