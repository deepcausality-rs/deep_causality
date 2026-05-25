/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Boundary-layer kernels for wall-bounded turbulent flow.
//!
//! Includes wall shear stress (Newtonian), friction velocity, viscous length
//! scale, dimensionless wall distance `y⁺`, viscous-sublayer and log-law
//! velocity profiles, and the skin-friction coefficient.
//!
//! Conventions: `WallShearStress<R>` stores the magnitude `|τ_w|`. The von
//! Kármán constant `κ ≈ 0.41` and the log-law constant `B ≈ 5.0` are taken
//! as inputs by `log_law_velocity_kernel` so the caller can pick the value
//! convention used by their reference data.

use crate::Density;
use crate::PhysicsError;
use crate::kernels::dynamics::quantities::{Length, Speed};
use crate::kernels::fluids::quantities::{KinematicViscosity, Viscosity, WallShearStress};
use deep_causality_num::RealField;

/// Newtonian wall shear stress magnitude `τ_w = μ · |∂u/∂y|_wall` (Pa).
///
/// `WallShearStress<R>` is a magnitude; this kernel always returns the
/// absolute value. Sign of the underlying gradient is the caller's concern.
///
/// Errors when the gradient input is non-finite (NaN/Inf) — `du_dy_wall`
/// is a raw `R` scalar with no upstream invariant, so the checked
/// constructor is used to reject `WallShearStress::NaN` rather than
/// admit it silently.
pub fn wall_shear_stress_newtonian_kernel<R>(
    mu: &Viscosity<R>,
    du_dy_wall: R,
) -> Result<WallShearStress<R>, PhysicsError>
where
    R: RealField,
{
    let abs_gradient = if du_dy_wall < R::zero() {
        -du_dy_wall
    } else {
        du_dy_wall
    };
    WallShearStress::new(mu.value() * abs_gradient)
}

/// Friction velocity `u_τ = √(τ_w / ρ)` (m/s).
///
/// Errors on `ρ = 0`.
pub fn friction_velocity_kernel<R>(
    tau_w: &WallShearStress<R>,
    rho: &Density<R>,
) -> Result<Speed<R>, PhysicsError>
where
    R: RealField,
{
    let r = rho.value();
    if r == R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "friction_velocity_kernel: density is zero".into(),
        ));
    }
    Speed::new((tau_w.value() / r).sqrt())
}

/// Viscous length scale `δ_ν = ν / u_τ` (m).
///
/// Errors on `u_τ = 0`.
pub fn viscous_length_scale_kernel<R>(
    nu: &KinematicViscosity<R>,
    u_tau: &Speed<R>,
) -> Result<Length<R>, PhysicsError>
where
    R: RealField,
{
    let u = u_tau.value();
    if u == R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "viscous_length_scale_kernel: friction velocity is zero".into(),
        ));
    }
    Length::new(nu.value() / u)
}

/// Dimensionless wall distance `y⁺ = y · u_τ / ν`.
///
/// Errors on `ν = 0`.
pub fn y_plus_kernel<R>(
    y: &Length<R>,
    u_tau: &Speed<R>,
    nu: &KinematicViscosity<R>,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let v = nu.value();
    if v == R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "y_plus_kernel: kinematic viscosity is zero".into(),
        ));
    }
    Ok(y.value() * u_tau.value() / v)
}

/// Viscous sublayer velocity profile `u⁺ = y⁺` (valid for `y⁺ ≲ 5`).
///
/// Trivial identity, exposed for symmetry with the log-law kernel and as a
/// concrete consumption point for the typed wall-distance value.
pub fn viscous_sublayer_velocity_kernel<R>(y_plus: R) -> R
where
    R: RealField,
{
    y_plus
}

/// Logarithmic law of the wall `u⁺ = (1/κ) · ln(y⁺) + B`.
///
/// Errors on `κ = 0` (division by zero) or `y⁺ ≤ 0` (log undefined).
/// Standard values: `κ ≈ 0.41`, `B ≈ 5.0`. Valid for roughly `30 ≲ y⁺ ≲ 300`.
pub fn log_law_velocity_kernel<R>(y_plus: R, kappa: R, b: R) -> Result<R, PhysicsError>
where
    R: RealField,
{
    if kappa == R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "log_law_velocity_kernel: kappa is zero".into(),
        ));
    }
    if y_plus <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "log_law_velocity_kernel: y_plus must be > 0".into(),
        ));
    }
    Ok(y_plus.ln() / kappa + b)
}

/// Skin-friction coefficient `C_f = τ_w / (0.5 · ρ · u_∞²)`.
///
/// Errors on `ρ = 0` or `u_∞ = 0`.
pub fn skin_friction_coefficient_kernel<R>(
    tau_w: &WallShearStress<R>,
    rho: &Density<R>,
    u_inf: &Speed<R>,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let r = rho.value();
    let u = u_inf.value();
    if r == R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "skin_friction_coefficient_kernel: density is zero".into(),
        ));
    }
    if u == R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "skin_friction_coefficient_kernel: u_inf is zero".into(),
        ));
    }
    // Two = 1 + 1 (avoids FromPrimitive bound).
    let two = R::one() + R::one();
    Ok(tau_w.value() * two / (r * u * u))
}
