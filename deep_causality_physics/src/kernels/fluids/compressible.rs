/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Compressible-flow thermodynamic kernels.
//!
//! Includes ideal-gas speed of sound, specific and total enthalpy, isentropic
//! stagnation (total) pressure / temperature, and the local entropy-production
//! rate for a Newtonian fluid with heat conduction.
//!
//! Conventions: SI units throughout. `γ` is the ratio of specific heats
//! (dimensionless); `R_specific` is the specific gas constant (J/(kg·K));
//! `c_p` is the specific heat at constant pressure (J/(kg·K)).

use crate::PhysicsError;
use crate::Temperature;
use crate::kernels::dynamics::quantities::Speed;
use crate::kernels::fluids::quantities::{
    Pressure, SpecificEnthalpy, Velocity3, VelocityGradient, ViscousStress,
};
use deep_causality_num::{FromPrimitive, RealField};

/// Ideal-gas speed of sound `a = √(γ · R_s · T)`.
///
/// Errors when `γ · R_s · T ≤ 0` or when [`Speed::new`] rejects the result.
pub fn speed_of_sound_ideal_gas_kernel<R>(
    gamma: R,
    r_specific: R,
    temperature: &Temperature<R>,
) -> Result<Speed<R>, PhysicsError>
where
    R: RealField,
{
    let arg = gamma * r_specific * temperature.value();
    if arg <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "speed_of_sound_ideal_gas_kernel: γ·R_s·T must be positive".into(),
        ));
    }
    Speed::new(arg.sqrt())
}

/// Specific enthalpy `h = c_p · T` (J/kg) for an ideal gas with constant `c_p`.
pub fn specific_enthalpy_kernel<R>(cp: R, temperature: &Temperature<R>) -> SpecificEnthalpy<R>
where
    R: RealField,
{
    SpecificEnthalpy::new_unchecked(cp * temperature.value())
}

/// Total (stagnation) enthalpy `h_0 = h + 0.5 · ‖u‖²` (J/kg).
pub fn total_enthalpy_kernel<R>(
    h: &SpecificEnthalpy<R>,
    u: &Velocity3<R>,
) -> Result<SpecificEnthalpy<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let u_raw = u.value();
    let speed_sq = u_raw[0] * u_raw[0] + u_raw[1] * u_raw[1] + u_raw[2] * u_raw[2];
    Ok(SpecificEnthalpy::new_unchecked(h.value() + half * speed_sq))
}

/// Isentropic stagnation pressure
/// `p_0 = p · (1 + (γ−1)/2 · M²)^(γ/(γ−1))`.
///
/// Errors when `γ ≤ 1` (the exponent diverges) or when [`Pressure::new`]
/// rejects the result.
pub fn total_pressure_isentropic_kernel<R>(
    p: &Pressure<R>,
    mach: R,
    gamma: R,
) -> Result<Pressure<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let one = R::one();
    if gamma <= one {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "total_pressure_isentropic_kernel: γ must be > 1".into(),
        ));
    }
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let base = one + (gamma - one) * half * mach * mach;
    if base <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "total_pressure_isentropic_kernel: base of exponent must be positive".into(),
        ));
    }
    let exponent = gamma / (gamma - one);
    Pressure::new(p.value() * base.powf(exponent))
}

/// Isentropic stagnation temperature
/// `T_0 = T · (1 + (γ−1)/2 · M²)`.
pub fn total_temperature_isentropic_kernel<R>(
    t: &Temperature<R>,
    mach: R,
    gamma: R,
) -> Result<Temperature<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let one = R::one();
    if gamma <= one {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "total_temperature_isentropic_kernel: γ must be > 1".into(),
        ));
    }
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let factor = one + (gamma - one) * half * mach * mach;
    Temperature::new(t.value() * factor)
}

/// Local entropy-production rate density (W/(m³·K)) for a Newtonian fluid:
/// `σ = Φ/T + κ · ‖∇T‖² / T²`
/// where `Φ = τ : ∇u` is viscous dissipation and `κ` is thermal conductivity.
///
/// For a Newtonian fluid both terms are non-negative (Clausius–Duhem
/// inequality). Errors when `T ≤ 0` (division by zero or negative absolute
/// temperature).
///
/// The `tau` argument is typed as [`ViscousStress<R>`], so the `σ ≥ 0`
/// second-law guarantee is preserved at the type level — the full Cauchy
/// stress `σ = −p I + τ` cannot be passed here without an explicit
/// conversion.
pub fn entropy_production_rate_kernel<R>(
    temperature: &Temperature<R>,
    tau: &ViscousStress<R>,
    grad_u: &VelocityGradient<R>,
    thermal_conductivity: R,
    grad_temperature: &[R; 3],
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let t = temperature.value();
    if t <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "entropy_production_rate_kernel: temperature must be > 0".into(),
        ));
    }
    // Fourier's law fixes the sign of κ ≥ 0 for any physically realisable
    // material; a negative thermal conductivity would have heat flowing
    // from cold to hot and would let the kernel return σ < 0, breaking
    // the second-law (non-negativity) contract advertised above.
    if thermal_conductivity < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "entropy_production_rate_kernel: thermal_conductivity must be ≥ 0".into(),
        ));
    }

    // Φ = τ : ∇u
    let tv = tau.value();
    let gv = grad_u.value();
    let phi = tv[0][0] * gv[0][0]
        + tv[0][1] * gv[0][1]
        + tv[0][2] * gv[0][2]
        + tv[1][0] * gv[1][0]
        + tv[1][1] * gv[1][1]
        + tv[1][2] * gv[1][2]
        + tv[2][0] * gv[2][0]
        + tv[2][1] * gv[2][1]
        + tv[2][2] * gv[2][2];

    let grad_t_norm_sq = grad_temperature[0] * grad_temperature[0]
        + grad_temperature[1] * grad_temperature[1]
        + grad_temperature[2] * grad_temperature[2];

    Ok(phi / t + thermal_conductivity * grad_t_norm_sq / (t * t))
}
