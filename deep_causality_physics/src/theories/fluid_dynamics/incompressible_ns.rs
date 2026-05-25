/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Incompressible Newtonian Navier–Stokes regime evaluator.
//!
//! Composes the pointwise kernels from [`crate::kernels::fluids::governing`]
//! into the Eulerian acceleration form of the incompressible NS momentum
//! equation. The continuity equation `∇·u = 0` is a constraint on the input
//! field and is not enforced by this kernel.
//!
//! Equation form (RHS of `∂u/∂t = …`):
//!
//! ```text
//! ∂u/∂t = − (u·∇)u − (1/ρ) ∇p + ν ∇²u + g
//! ```
//!
//! where `g` is the body-force-per-unit-mass (e.g. gravity). Sign convention:
//! positive `g_z` accelerates the fluid in `+z`. Errors propagate from
//! [`pressure_gradient_force_kernel`] when `ρ = 0`.

use crate::PhysicsError;
use crate::kernels::fluids::governing::{
    convective_acceleration_kernel, pressure_gradient_force_kernel, viscous_diffusion_kernel,
};
use crate::kernels::fluids::quantities::{
    AccelerationVector, Density, KinematicViscosity, Velocity3, VelocityGradient,
};
use deep_causality_num::RealField;

/// Pointwise RHS of the incompressible Newtonian momentum equation.
///
/// `∂u/∂t = − (u·∇)u − (1/ρ) ∇p + ν ∇²u + g`
///
/// - `u`            — velocity (m/s)
/// - `grad_u`       — velocity Jacobian, `[i][j] = ∂u_i/∂x_j`
/// - `laplacian_u`  — `∇²u` (1/(m·s))
/// - `grad_p`       — pressure gradient (Pa/m)
/// - `rho`          — fluid density (kg/m³); errors when zero
/// - `nu`           — kinematic viscosity (m²/s)
/// - `body_force_per_mass` — body acceleration (m/s²), e.g. gravity
///
/// Returns the Eulerian acceleration `∂u/∂t` at the sample point.
pub fn incompressible_ns_rhs_kernel<R>(
    u: &Velocity3<R>,
    grad_u: &VelocityGradient<R>,
    laplacian_u: &[R; 3],
    grad_p: &[R; 3],
    rho: &Density<R>,
    nu: &KinematicViscosity<R>,
    body_force_per_mass: &AccelerationVector<R>,
) -> Result<AccelerationVector<R>, PhysicsError>
where
    R: RealField,
{
    let conv = convective_acceleration_kernel(u, grad_u).into_inner();
    let press = pressure_gradient_force_kernel(rho, grad_p)?.into_inner();
    let visc = viscous_diffusion_kernel(nu, laplacian_u).into_inner();
    let g = body_force_per_mass.value();

    Ok(AccelerationVector::new_unchecked([
        -conv[0] + press[0] + visc[0] + g[0],
        -conv[1] + press[1] + visc[1] + g[1],
        -conv[2] + press[2] + visc[2] + g[2],
    ]))
}
