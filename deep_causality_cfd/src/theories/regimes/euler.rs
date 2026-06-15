/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Euler regime evaluator (inviscid limit of Navier-Stokes).
//!
//! Pointwise RHS of the Euler momentum equation in Eulerian acceleration form:
//!
//! ```text
//! ∂u/∂t = − (u·∇)u − (1/ρ) ∇p + g
//! ```
//!
//! No viscosity input: the viscous diffusion term `ν ∇²u` is identically zero
//! in this regime, so the signature omits both `ν` and `∇²u`. Errors propagate
//! from [`pressure_gradient_force_kernel`] when `ρ = 0`.

use deep_causality_num::RealField;
use deep_causality_physics::PhysicsError;
use deep_causality_physics::{AccelerationVector, Density, Velocity3, VelocityGradient};
use deep_causality_physics::{convective_acceleration_kernel, pressure_gradient_force_kernel};

/// Pointwise RHS of the Euler momentum equation (inviscid).
///
/// `∂u/∂t = − (u·∇)u − (1/ρ) ∇p + g`
///
/// - `u`            — velocity (m/s)
/// - `grad_u`       — velocity Jacobian, `[i][j] = ∂u_i/∂x_j`
/// - `grad_p`       — pressure gradient (Pa/m)
/// - `rho`          — fluid density (kg/m³); errors when zero
/// - `body_force_per_mass` — body acceleration (m/s²)
pub fn euler_momentum_rhs_kernel<R>(
    u: &Velocity3<R>,
    grad_u: &VelocityGradient<R>,
    grad_p: &[R; 3],
    rho: &Density<R>,
    body_force_per_mass: &AccelerationVector<R>,
) -> Result<AccelerationVector<R>, PhysicsError>
where
    R: RealField,
{
    let conv = convective_acceleration_kernel(u, grad_u).into_inner();
    let press = pressure_gradient_force_kernel(rho, grad_p)?.into_inner();
    let g = body_force_per_mass.value();

    Ok(AccelerationVector::new_unchecked([
        -conv[0] + press[0] + g[0],
        -conv[1] + press[1] + g[1],
        -conv[2] + press[2] + g[2],
    ]))
}
