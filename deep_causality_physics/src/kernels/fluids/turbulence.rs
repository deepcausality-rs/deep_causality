/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Turbulence-quantity kernels for fluid mechanics.
//!
//! Includes pointwise builders for turbulent kinetic energy, mean dissipation
//! rate, the Kolmogorov micro-scales (length, time, velocity), the Taylor
//! microscale, the integral length scale, the Reynolds-stress tensor, and a
//! Boussinesq-closure eddy viscosity.
//!
//! Sign convention: the Reynolds-stress tensor here is `R_ij = ⟨u'_i u'_j⟩`
//! (positive-semidefinite on the diagonal). The Boussinesq-closure relation
//! used by `eddy_viscosity_boussinesq_kernel` is
//! `R_ij − (2/3) k δ_ij = −2 ν_t S_ij`, solved by least-squares.

use crate::PhysicsError;
use crate::{
    KinematicViscosity, ReynoldsStress, StrainRateTensor, Velocity3, VelocityGradient, Viscosity,
};
use crate::{Length, Speed};
use deep_causality_num::{FromPrimitive, RealField};

/// Turbulent kinetic energy `k = 0.5 · ⟨u' · u'⟩`.
///
/// Caller is responsible for supplying the already-averaged fluctuation
/// velocity (or an instantaneous one, in which case the output is the
/// instantaneous kinetic energy density per unit mass).
pub fn turbulent_kinetic_energy_kernel<R>(u_prime: &Velocity3<R>) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let u = u_prime.value();
    Ok(half * (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]))
}

/// Dissipation rate `ε = 2 ν · S':S' = 2 ν · Σ_{i,j} S'_{ij}²`.
///
/// `S'` is the symmetric part of the fluctuation velocity gradient, computed
/// internally from `grad_u_prime`. Non-negative by construction.
pub fn dissipation_rate_kernel<R>(
    nu: &KinematicViscosity<R>,
    grad_u_prime: &VelocityGradient<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let g = grad_u_prime.value();

    // S'_ij = 0.5 (g_ij + g_ji); sum of squares, unrolled 3x3.
    let s00 = g[0][0];
    let s11 = g[1][1];
    let s22 = g[2][2];
    let s01 = half * (g[0][1] + g[1][0]);
    let s02 = half * (g[0][2] + g[2][0]);
    let s12 = half * (g[1][2] + g[2][1]);
    // S is symmetric, so off-diagonal contributions appear twice in the
    // sum over all (i, j).
    let sum_sq = s00 * s00 + s11 * s11 + s22 * s22 + (s01 * s01 + s02 * s02 + s12 * s12) * two;
    Ok(two * nu.value() * sum_sq)
}

#[inline]
fn require_positive<R: RealField>(val: R, ctx: &str) -> Result<R, PhysicsError> {
    if val <= R::zero() {
        Err(PhysicsError::PhysicalInvariantBroken(format!(
            "{}: argument must be > 0",
            ctx
        )))
    } else {
        Ok(val)
    }
}

/// Kolmogorov length scale `η = (ν³ / ε)^(1/4)` (m).
///
/// Errors when `ν ≤ 0` or `ε ≤ 0` (the kernel needs both positive to compute
/// a finite physical length).
pub fn kolmogorov_length_kernel<R>(
    nu: &KinematicViscosity<R>,
    epsilon: R,
) -> Result<Length<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let v = require_positive(nu.value(), "kolmogorov_length_kernel: nu")?;
    let e = require_positive(epsilon, "kolmogorov_length_kernel: epsilon")?;
    let quarter = R::from_f64(0.25)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.25) failed".into()))?;
    let eta = (v * v * v / e).powf(quarter);
    Length::new(eta)
}

/// Kolmogorov time scale `τ_η = (ν / ε)^(1/2)` (s).
pub fn kolmogorov_time_kernel<R>(nu: &KinematicViscosity<R>, epsilon: R) -> Result<R, PhysicsError>
where
    R: RealField,
{
    let v = require_positive(nu.value(), "kolmogorov_time_kernel: nu")?;
    let e = require_positive(epsilon, "kolmogorov_time_kernel: epsilon")?;
    Ok((v / e).sqrt())
}

/// Kolmogorov velocity scale `u_η = (ν · ε)^(1/4)` (m/s).
pub fn kolmogorov_velocity_kernel<R>(
    nu: &KinematicViscosity<R>,
    epsilon: R,
) -> Result<Speed<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let v = require_positive(nu.value(), "kolmogorov_velocity_kernel: nu")?;
    let e = require_positive(epsilon, "kolmogorov_velocity_kernel: epsilon")?;
    let quarter = R::from_f64(0.25)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.25) failed".into()))?;
    Speed::new((v * e).powf(quarter))
}

/// Taylor microscale `λ = √(15 · ν · k / ε)` (m).
pub fn taylor_microscale_kernel<R>(
    k_energy: R,
    epsilon: R,
    nu: &KinematicViscosity<R>,
) -> Result<Length<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let v = require_positive(nu.value(), "taylor_microscale_kernel: nu")?;
    let e = require_positive(epsilon, "taylor_microscale_kernel: epsilon")?;
    if k_energy < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "taylor_microscale_kernel: k_energy must be non-negative".into(),
        ));
    }
    let fifteen = R::from_f64(15.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(15.0) failed".into()))?;
    Length::new((fifteen * v * k_energy / e).sqrt())
}

/// Integral length scale `L = k^(3/2) / ε` (m).
pub fn integral_length_scale_kernel<R>(k_energy: R, epsilon: R) -> Result<Length<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let e = require_positive(epsilon, "integral_length_scale_kernel: epsilon")?;
    if k_energy < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "integral_length_scale_kernel: k_energy must be non-negative".into(),
        ));
    }
    let three_halves = R::from_f64(1.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(1.5) failed".into()))?;
    Length::new(k_energy.powf(three_halves) / e)
}

/// Reynolds-stress tensor `R_ij = ⟨u'_i u'_j⟩` packaged as a [`ReynoldsStress<R>`].
///
/// The input is the already-averaged outer-product tensor (symmetric by
/// physical construction); the kernel is a typed pass-through that pins the
/// symmetric-tensor invariant in the output. Diagonal entries are
/// non-negative (variances).
///
/// The dedicated [`ReynoldsStress<R>`] output type distinguishes this from
/// viscous stress at the type level — it cannot be passed into
/// `viscous_dissipation_rate_kernel` or `entropy_production_rate_kernel`
/// without an explicit conversion.
pub fn reynolds_stress_kernel<R>(u_prime_outer_u_prime: &StrainRateTensor<R>) -> ReynoldsStress<R>
where
    R: RealField,
{
    ReynoldsStress::new_unchecked(*u_prime_outer_u_prime.value())
}

/// Boussinesq-closure eddy viscosity `ν_t`.
///
/// Solves the Boussinesq relation
/// `R_ij − (2/3) k δ_ij = −2 ν_t S_ij`
/// by least-squares contraction:
/// `ν_t = −(R^dev : S) / (2 · S : S)` where `R^dev = R − (2/3) k I`.
///
/// Errors when the mean strain is identically zero (eddy viscosity is
/// undefined when the closure equation has no signal) or when the resulting
/// `ν_t` is negative or non-finite (in which case `Viscosity::new` rejects).
pub fn eddy_viscosity_boussinesq_kernel<R>(
    reynolds_stress: &ReynoldsStress<R>,
    strain_rate_mean: &StrainRateTensor<R>,
    k_energy: R,
) -> Result<Viscosity<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    let two_thirds = R::from_f64(2.0 / 3.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0/3.0) failed".into()))?;

    let r = reynolds_stress.value();
    let s = strain_rate_mean.value();

    // Subtract isotropic part: R^dev = R - (2/3) k I (only diagonal modified).
    let mut r_dev = *r;
    let iso = two_thirds * k_energy;
    r_dev[0][0] -= iso;
    r_dev[1][1] -= iso;
    r_dev[2][2] -= iso;

    // R^dev : S = Σ_{i,j} R^dev_ij · S_ij, unrolled.
    let mut rs = R::zero();
    let mut ss = R::zero();
    for (r_row, s_row) in r_dev.iter().zip(s.iter()) {
        for (r_ij, s_ij) in r_row.iter().zip(s_row.iter()) {
            rs += *r_ij * *s_ij;
            ss += *s_ij * *s_ij;
        }
    }

    if ss == R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "eddy_viscosity_boussinesq_kernel: strain rate is zero; eddy viscosity undefined"
                .into(),
        ));
    }
    Viscosity::new(-rs / (two * ss))
}
