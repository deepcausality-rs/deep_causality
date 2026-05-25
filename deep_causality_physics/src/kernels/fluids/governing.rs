/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Pointwise governing-equation kernels for fluid mechanics.
//!
//! These kernels evaluate the RHS contributions of the classical conservation
//! laws (mass, momentum, vorticity, scalar transport, energy building blocks).
//! All kernels are stateless and side-effect-free. Velocity-gradient inputs
//! follow the Jacobian convention `[i][j] = ∂u_i/∂x_j`, pinned by
//! [`VelocityGradient<R>`].
//!
//! Convention note: the convective term `(u·∇)u` is not Galilean invariant
//! in isolation. The full material derivative `Du/Dt = ∂u/∂t + (u·∇)u` is.
//! These kernels return the spatial RHS only; the explicit time-derivative
//! term is the caller's concern.

use crate::PhysicsError;
use crate::kernels::fluids::quantities::{
    AccelerationVector, Density, KinematicViscosity, Pressure, Velocity3, VelocityGradient,
    ViscousStress, VorticityVector,
};
use deep_causality_num::{FromPrimitive, RealField};

// =============================================================================
// Momentum equation building blocks
// =============================================================================

/// Convective acceleration `(u·∇)u`.
///
/// Component `i`: `Σ_j u_j · ∂u_i/∂x_j = Σ_j u_j · grad_u[i][j]`.
///
/// Units: `(m/s) · (1/s) = m/s²` → returned as [`AccelerationVector<R>`].
///
/// Not Galilean invariant in isolation: shifting `u → u + c` shifts the
/// result by `grad_u · c` (this is the property the test scenario verifies).
pub fn convective_acceleration_kernel<R>(
    u: &Velocity3<R>,
    grad_u: &VelocityGradient<R>,
) -> AccelerationVector<R>
where
    R: RealField,
{
    let u_raw = u.value();
    let g = grad_u.value();
    AccelerationVector::new_unchecked([
        u_raw[0] * g[0][0] + u_raw[1] * g[0][1] + u_raw[2] * g[0][2],
        u_raw[0] * g[1][0] + u_raw[1] * g[1][1] + u_raw[2] * g[1][2],
        u_raw[0] * g[2][0] + u_raw[1] * g[2][1] + u_raw[2] * g[2][2],
    ])
}

/// Viscous diffusion acceleration `ν · ∇²u`.
///
/// Linear in both `nu` and `laplacian_u`. Returns the component-wise product
/// scaled by kinematic viscosity. Vanishes in the inviscid limit `ν = 0`.
pub fn viscous_diffusion_kernel<R>(
    nu: &KinematicViscosity<R>,
    laplacian_u: &[R; 3],
) -> AccelerationVector<R>
where
    R: RealField,
{
    let v = nu.value();
    AccelerationVector::new_unchecked([v * laplacian_u[0], v * laplacian_u[1], v * laplacian_u[2]])
}

/// Pressure gradient force per unit mass `−(1/ρ) · ∇p`.
///
/// Errors when `ρ = 0` to avoid division by zero. `Density::new` already
/// enforces `ρ ≥ 0`, so the only failure mode here is the boundary case.
pub fn pressure_gradient_force_kernel<R>(
    rho: &Density<R>,
    grad_p: &[R; 3],
) -> Result<AccelerationVector<R>, PhysicsError>
where
    R: RealField,
{
    let r = rho.value();
    if r == R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "pressure_gradient_force_kernel: density is zero".into(),
        ));
    }
    let inv_rho = R::one() / r;
    Ok(AccelerationVector::new_unchecked([
        -inv_rho * grad_p[0],
        -inv_rho * grad_p[1],
        -inv_rho * grad_p[2],
    ]))
}

// =============================================================================
// Continuity equation
// =============================================================================

/// Continuity equation RHS: `∂ρ/∂t = −∇·(ρu) = −(u·∇ρ + ρ · ∇·u)`.
///
/// Returns the scalar RHS. Reduces to `0` for incompressible flow when
/// `grad_rho = 0` and `div_u = 0`.
pub fn continuity_rhs_kernel<R>(
    rho: &Density<R>,
    u: &Velocity3<R>,
    grad_rho: &[R; 3],
    div_u: R,
) -> R
where
    R: RealField,
{
    let r = rho.value();
    let u_raw = u.value();
    let u_dot_grad_rho = u_raw[0] * grad_rho[0] + u_raw[1] * grad_rho[1] + u_raw[2] * grad_rho[2];
    -(u_dot_grad_rho + r * div_u)
}

// =============================================================================
// Vorticity transport
// =============================================================================

/// Vorticity transport RHS: `−(u·∇)ω + (ω·∇)u + ν · ∇²ω`.
///
/// Components:
/// - Advection of vorticity: `(u·∇)ω_i = Σ_j u_j · grad_omega[i][j]`.
/// - Vortex stretching:        `(ω·∇)u_i = Σ_j ω_j · grad_u[i][j]`.
/// - Viscous diffusion:        `ν · laplacian_omega_i`.
///
/// At `ν = 0` this reduces to the inviscid Helmholtz vorticity equation
/// `−(u·∇)ω + (ω·∇)u`.
pub fn vorticity_transport_kernel<R>(
    omega: &VorticityVector<R>,
    u: &Velocity3<R>,
    grad_u: &VelocityGradient<R>,
    grad_omega: &[[R; 3]; 3],
    laplacian_omega: &[R; 3],
    nu: &KinematicViscosity<R>,
) -> AccelerationVector<R>
where
    R: RealField,
{
    let w = omega.value();
    let u_raw = u.value();
    let gu = grad_u.value();
    let v = nu.value();

    // Advection: (u · ∇) ω_i = Σ_j u_j · grad_omega[i][j]
    let adv = [
        u_raw[0] * grad_omega[0][0] + u_raw[1] * grad_omega[0][1] + u_raw[2] * grad_omega[0][2],
        u_raw[0] * grad_omega[1][0] + u_raw[1] * grad_omega[1][1] + u_raw[2] * grad_omega[1][2],
        u_raw[0] * grad_omega[2][0] + u_raw[1] * grad_omega[2][1] + u_raw[2] * grad_omega[2][2],
    ];

    // Vortex stretching: (ω · ∇) u_i = Σ_j ω_j · grad_u[i][j]
    let stretch = [
        w[0] * gu[0][0] + w[1] * gu[0][1] + w[2] * gu[0][2],
        w[0] * gu[1][0] + w[1] * gu[1][1] + w[2] * gu[1][2],
        w[0] * gu[2][0] + w[1] * gu[2][1] + w[2] * gu[2][2],
    ];

    AccelerationVector::new_unchecked([
        -adv[0] + stretch[0] + v * laplacian_omega[0],
        -adv[1] + stretch[1] + v * laplacian_omega[1],
        -adv[2] + stretch[2] + v * laplacian_omega[2],
    ])
}

// =============================================================================
// Scalar advection-diffusion
// =============================================================================

/// Generic scalar transport RHS: `−u · ∇φ + D · ∇²φ + S`.
///
/// Reduces to pure advection when `diffusivity = 0`, to pure diffusion when
/// `u = 0`, and to source-only when both vanish.
pub fn scalar_advection_diffusion_kernel<R>(
    u: &Velocity3<R>,
    grad_phi: &[R; 3],
    laplacian_phi: R,
    diffusivity: R,
    source: R,
) -> R
where
    R: RealField,
{
    let u_raw = u.value();
    let advection = u_raw[0] * grad_phi[0] + u_raw[1] * grad_phi[1] + u_raw[2] * grad_phi[2];
    -advection + diffusivity * laplacian_phi + source
}

// =============================================================================
// Energy equation building blocks
// =============================================================================

/// Kinetic energy density `ρ · 0.5 · ‖u‖²` (J/m³).
///
/// Non-negative for any finite inputs (density invariant `ρ ≥ 0`, squared norm
/// non-negative).
pub fn kinetic_energy_density_kernel<R>(
    rho: &Density<R>,
    u: &Velocity3<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let u_raw = u.value();
    let speed_sq = u_raw[0] * u_raw[0] + u_raw[1] * u_raw[1] + u_raw[2] * u_raw[2];
    Ok(rho.value() * half * speed_sq)
}

/// Viscous dissipation rate `Φ = τ : ∇u` (W/m³).
///
/// Local rate of mechanical energy converted to heat. For a Newtonian fluid
/// (`τ` constructed via `newtonian_viscous_stress_kernel`), `Φ ≥ 0` by the
/// Clausius–Duhem inequality. Returned as a raw scalar; sign-checking is the
/// caller's responsibility when the input `tau` is not guaranteed Newtonian.
///
/// The `tau` argument is typed as [`ViscousStress<R>`] (not [`CauchyStress<R>`]),
/// so the `Φ ≥ 0` Clausius–Duhem positivity guarantee is preserved at the
/// type level rather than relying on docstring discipline.
///
/// Tensor double-contraction:
/// `τ : ∇u = Σ_i Σ_j τ_ij · grad_u[i][j]`.
pub fn viscous_dissipation_rate_kernel<R>(tau: &ViscousStress<R>, grad_u: &VelocityGradient<R>) -> R
where
    R: RealField,
{
    let t = tau.value();
    let g = grad_u.value();
    t[0][0] * g[0][0]
        + t[0][1] * g[0][1]
        + t[0][2] * g[0][2]
        + t[1][0] * g[1][0]
        + t[1][1] * g[1][1]
        + t[1][2] * g[1][2]
        + t[2][0] * g[2][0]
        + t[2][1] * g[2][1]
        + t[2][2] * g[2][2]
}

/// Reversible pressure work `p · ∇·u` (W/m³).
///
/// Positive when the flow expands at positive pressure (work done by the
/// fluid on its surroundings); negative when the flow compresses.
pub fn pressure_work_kernel<R>(p: &Pressure<R>, div_u: R) -> R
where
    R: RealField,
{
    p.value() * div_u
}
