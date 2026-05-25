/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Dimensionless number kernels for fluid mechanics.
//!
//! Each kernel computes one canonical dimensionless number from SI inputs.
//! Where the formula has a possible divide-by-zero or non-physical-input
//! case, the kernel returns `Result<R, PhysicsError>`; otherwise it returns
//! `R` directly.
//!
//! Identities maintained across the surface:
//! - `Pe = Re · Pr`
//! - `Ra = Gr · Pr`
//! - `Le = α / D = Sc / Pr`

use crate::PhysicsError;
use crate::kernels::dynamics::quantities::{Length, Speed};
use crate::kernels::fluids::quantities::{Density, KinematicViscosity, Viscosity};
use deep_causality_num::RealField;

#[inline]
fn ensure_nonzero<R: RealField>(val: R, ctx: &str) -> Result<R, PhysicsError> {
    if val == R::zero() {
        Err(PhysicsError::PhysicalInvariantBroken(format!(
            "{}: divisor is zero",
            ctx
        )))
    } else {
        Ok(val)
    }
}

/// Reynolds number `Re = u · L / ν`. Ratio of inertial to viscous forces.
pub fn reynolds_number_kernel<R>(
    u: &Speed<R>,
    length: &Length<R>,
    nu: &KinematicViscosity<R>,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let v = ensure_nonzero(nu.value(), "reynolds_number_kernel")?;
    Ok(u.value() * length.value() / v)
}

/// Mach number `M = u / a`. Flow speed in units of the local sound speed.
pub fn mach_number_kernel<R>(u: &Speed<R>, sound_speed: &Speed<R>) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let a = ensure_nonzero(sound_speed.value(), "mach_number_kernel")?;
    Ok(u.value() / a)
}

/// Froude number `Fr = u / √(g · L)`. Inertial to gravitational forces.
pub fn froude_number_kernel<R>(
    u: &Speed<R>,
    gravity: R,
    length: &Length<R>,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let g_l = gravity * length.value();
    if g_l <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "froude_number_kernel: gravity · length must be positive".into(),
        ));
    }
    Ok(u.value() / g_l.sqrt())
}

/// Weber number `We = ρ · u² · L / σ`. Inertial to surface-tension forces.
pub fn weber_number_kernel<R>(
    rho: &Density<R>,
    u: &Speed<R>,
    length: &Length<R>,
    surface_tension: R,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let sigma = ensure_nonzero(surface_tension, "weber_number_kernel")?;
    let u_val = u.value();
    Ok(rho.value() * u_val * u_val * length.value() / sigma)
}

/// Prandtl number `Pr = ν / α`. Momentum diffusivity to thermal diffusivity.
pub fn prandtl_number_kernel<R>(
    nu: &KinematicViscosity<R>,
    thermal_diffusivity: R,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let alpha = ensure_nonzero(thermal_diffusivity, "prandtl_number_kernel")?;
    Ok(nu.value() / alpha)
}

/// Peclet number (thermal) `Pe = u · L / α`. Convective to diffusive heat transport.
///
/// Identity: `Pe = Re · Pr`.
pub fn peclet_number_kernel<R>(
    u: &Speed<R>,
    length: &Length<R>,
    thermal_diffusivity: R,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let alpha = ensure_nonzero(thermal_diffusivity, "peclet_number_kernel")?;
    Ok(u.value() * length.value() / alpha)
}

/// Strouhal number `Sr = f · L / u`. Oscillation frequency to flow.
pub fn strouhal_number_kernel<R>(
    frequency: R,
    length: &Length<R>,
    u: &Speed<R>,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let u_val = ensure_nonzero(u.value(), "strouhal_number_kernel")?;
    Ok(frequency * length.value() / u_val)
}

/// Knudsen number `Kn = λ / L`. Mean free path to characteristic length.
pub fn knudsen_number_kernel<R>(mean_free_path: R, length: &Length<R>) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let l = ensure_nonzero(length.value(), "knudsen_number_kernel")?;
    Ok(mean_free_path / l)
}

/// Richardson number `Ri = g · β · ΔT · L / u²`. Buoyancy to shear forces.
pub fn richardson_number_kernel<R>(
    gravity: R,
    expansion_coefficient: R,
    delta_temperature: R,
    length: &Length<R>,
    u: &Speed<R>,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let u_val = ensure_nonzero(u.value(), "richardson_number_kernel")?;
    Ok(gravity * expansion_coefficient * delta_temperature * length.value() / (u_val * u_val))
}

/// Rayleigh number `Ra = g · β · ΔT · L³ / (ν · α)`. Drives free convection.
///
/// Identity: `Ra = Gr · Pr`.
pub fn rayleigh_number_kernel<R>(
    gravity: R,
    expansion_coefficient: R,
    delta_temperature: R,
    length: &Length<R>,
    nu: &KinematicViscosity<R>,
    thermal_diffusivity: R,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let v = ensure_nonzero(nu.value(), "rayleigh_number_kernel: nu")?;
    let alpha = ensure_nonzero(thermal_diffusivity, "rayleigh_number_kernel: alpha")?;
    let l = length.value();
    Ok(gravity * expansion_coefficient * delta_temperature * l * l * l / (v * alpha))
}

/// Grashof number `Gr = g · β · ΔT · L³ / ν²`. Buoyancy to viscous forces.
pub fn grashof_number_kernel<R>(
    gravity: R,
    expansion_coefficient: R,
    delta_temperature: R,
    length: &Length<R>,
    nu: &KinematicViscosity<R>,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let v = ensure_nonzero(nu.value(), "grashof_number_kernel")?;
    let l = length.value();
    Ok(gravity * expansion_coefficient * delta_temperature * l * l * l / (v * v))
}

/// Eckert number `Ec = u² / (c_p · ΔT)`. Kinetic energy to enthalpy difference.
pub fn eckert_number_kernel<R>(
    u: &Speed<R>,
    specific_heat: R,
    delta_temperature: R,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let denom = ensure_nonzero(
        specific_heat * delta_temperature,
        "eckert_number_kernel: c_p · ΔT",
    )?;
    let u_val = u.value();
    Ok(u_val * u_val / denom)
}

/// Schmidt number `Sc = ν / D`. Momentum diffusivity to mass diffusivity.
pub fn schmidt_number_kernel<R>(
    nu: &KinematicViscosity<R>,
    mass_diffusivity: R,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let d = ensure_nonzero(mass_diffusivity, "schmidt_number_kernel")?;
    Ok(nu.value() / d)
}

/// Lewis number `Le = α / D`. Thermal to mass diffusivity.
///
/// Identity: `Le = Sc / Pr`.
pub fn lewis_number_kernel<R>(
    thermal_diffusivity: R,
    mass_diffusivity: R,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let d = ensure_nonzero(mass_diffusivity, "lewis_number_kernel")?;
    Ok(thermal_diffusivity / d)
}

/// Particle Stokes number `St = τ_p · u / L`.
///
/// Ratio of particle response time to flow time scale. `St ≫ 1`: particle
/// trajectories decouple from flow; `St ≪ 1`: particles follow streamlines.
pub fn particle_stokes_number_kernel<R>(
    particle_relaxation_time: R,
    u: &Speed<R>,
    length: &Length<R>,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let l = ensure_nonzero(length.value(), "particle_stokes_number_kernel")?;
    Ok(particle_relaxation_time * u.value() / l)
}

/// Capillary number `Ca = μ · u / σ`. Viscous to surface-tension forces.
pub fn capillary_number_kernel<R>(
    mu: &Viscosity<R>,
    u: &Speed<R>,
    surface_tension: R,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let sigma = ensure_nonzero(surface_tension, "capillary_number_kernel")?;
    Ok(mu.value() * u.value() / sigma)
}

/// Bond (Eötvös) number `Bo = ρ · g · L² / σ`. Gravity to surface tension.
pub fn bond_number_kernel<R>(
    rho: &Density<R>,
    gravity: R,
    length: &Length<R>,
    surface_tension: R,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let sigma = ensure_nonzero(surface_tension, "bond_number_kernel")?;
    let l = length.value();
    Ok(rho.value() * gravity * l * l / sigma)
}

/// Nusselt number `Nu = h · L / k`. Convective to conductive heat transfer.
pub fn nusselt_number_kernel<R>(
    heat_transfer_coefficient: R,
    length: &Length<R>,
    thermal_conductivity: R,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let k = ensure_nonzero(thermal_conductivity, "nusselt_number_kernel")?;
    Ok(heat_transfer_coefficient * length.value() / k)
}
