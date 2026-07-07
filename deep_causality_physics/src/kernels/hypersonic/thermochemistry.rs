/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Park two-temperature thermochemistry kernels: vibrational relaxation
//! (Landau–Teller with a Millikan–White relaxation time, integrated by the
//! closed-form LER exponential) and the Arrhenius reaction-rate coefficient.

use crate::constants::{
    MILLIKAN_WHITE_A_COEFFICIENT, MILLIKAN_WHITE_LOG_OFFSET, MILLIKAN_WHITE_MU_OFFSET,
};
use crate::{PhysicsError, ReactionRate, Temperature, VibrationalTemperature};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Vibrational relaxation toward the translational temperature, integrated over
/// `dt` by the **closed-form Lagging-Equilibrium Relaxation (LER) exponential**
///
/// $$ T_{ve}(t+\Delta t) = T_{tr} - (T_{tr} - T_{ve})\,e^{-\Delta t / \tau_{vt}} $$
///
/// The relaxation time `τ_vt` is the Landau–Teller / Millikan–White correlation
///
/// $$ \tau_{vt}\,P = \exp\!\big[A_{sr}\,(T^{-1/3} - B\,\mu^{1/4}) - C\big], \quad
///    A_{sr} = a\,\mu^{1/2}\,\theta_v^{4/3} $$
///
/// (`P` in atm, `τ_vt` in s). The kernel returns the **integrated increment**
/// (the new `T_ve`), not a rate — unconditionally stable under stiffness and
/// exact on the linear relaxation, with `T_ve → T_tr` as `τ_vt → 0`.
///
/// # Arguments
/// * `t_ve` — current vibrational temperature `T_ve`.
/// * `t_tr` — target translational temperature `T_tr`.
/// * `pressure_atm` — pressure `P` in atm.
/// * `reduced_mass_amu` — reduced mass `μ_sr` of the colliding pair, in amu.
/// * `theta_vib` — characteristic vibrational temperature `θ_v`, in K.
/// * `dt` — timestep `Δt`, in s.
///
/// # References
/// * Millikan & White, "Systematics of Vibrational Relaxation," J. Chem. Phys.
///   39, 3209 (1963).
/// * Park, "Nonequilibrium Hypersonic Aerothermodynamics," Wiley (1990).
///   (The `1.16e-3` / `18.42` natural-log constants are the base-10 `5.0e-4` /
///   `8.00` originals scaled by `ln 10`.)
pub fn vibrational_relaxation_kernel<R>(
    t_ve: VibrationalTemperature<R>,
    t_tr: Temperature<R>,
    pressure_atm: R,
    reduced_mass_amu: R,
    theta_vib: R,
    dt: R,
) -> Result<VibrationalTemperature<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if pressure_atm <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Pressure must be positive for Millikan–White relaxation".into(),
        ));
    }
    if reduced_mass_amu <= R::zero() || theta_vib <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Reduced mass and vibrational temperature must be positive".into(),
        ));
    }
    if dt < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Timestep must be non-negative".into(),
        ));
    }
    let t = t_tr.value();
    if t <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Translational temperature must be positive".into(),
        ));
    }

    let a = R::from_f64(MILLIKAN_WHITE_A_COEFFICIENT).ok_or_else(|| {
        PhysicsError::NumericalInstability(
            "R::from_f64(MILLIKAN_WHITE_A_COEFFICIENT) failed".into(),
        )
    })?;
    let b = R::from_f64(MILLIKAN_WHITE_MU_OFFSET).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(MILLIKAN_WHITE_MU_OFFSET) failed".into())
    })?;
    let c = R::from_f64(MILLIKAN_WHITE_LOG_OFFSET).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(MILLIKAN_WHITE_LOG_OFFSET) failed".into())
    })?;
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let quarter = R::from_f64(0.25)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.25) failed".into()))?;
    let neg_third = R::from_f64(-1.0 / 3.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(-1/3) failed".into()))?;
    let four_thirds = R::from_f64(4.0 / 3.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(4/3) failed".into()))?;

    let a_sr = a * reduced_mass_amu.powf(half) * theta_vib.powf(four_thirds);
    let exponent = a_sr * (t.powf(neg_third) - b * reduced_mass_amu.powf(quarter)) - c;
    let tau = exponent.exp() / pressure_atm;
    if tau <= R::zero() || !tau.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Non-physical Millikan–White relaxation time".into(),
        ));
    }

    // LER closed-form exponential update (exact, unconditionally stable).
    let decay = (-(dt / tau)).exp();
    let t_ve_new = t - (t - t_ve.value()) * decay;
    VibrationalTemperature::new(t_ve_new)
}

/// Arrhenius reaction-rate coefficient in the Park / Gupta form
///
/// $$ k(T) = C_f\,T^{\eta}\,\exp(-\theta_d / T) $$
///
/// where `θ_d` is the activation (characteristic) temperature `E_a / k_B`. Used
/// for both forward and backward rates (pass the corresponding `C_f, η, θ_d`),
/// and for the dominant associative-ionization channel N + O → NO⁺ + e⁻ that
/// grounds the ionization relaxation time `τ_ion = 1 / (k_f · [M])`.
///
/// # Arguments
/// * `temperature` — rate-controlling temperature `T` (K).
/// * `prefactor` — `C_f`.
/// * `exponent` — `η`.
/// * `activation_temp` — `θ_d = E_a / k_B` (K).
///
/// # References
/// * Gupta, Yos, Thompson & Lee, NASA RP-1232 (1990), Table II, eq. 3a
///   (`papers/gupta_1990_nasa_rp1232.pdf`).
/// * Park, J. Thermophys. Heat Transfer 7(3):385 (1993).
pub fn arrhenius_rate_kernel<R>(
    temperature: Temperature<R>,
    prefactor: R,
    exponent: R,
    activation_temp: R,
) -> Result<ReactionRate<R>, PhysicsError>
where
    R: RealField,
{
    let t = temperature.value();
    if t <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Temperature must be positive for an Arrhenius rate".into(),
        ));
    }
    if prefactor < R::zero() || activation_temp < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Arrhenius prefactor and activation temperature must be non-negative".into(),
        ));
    }
    let rate = prefactor * t.powf(exponent) * (-(activation_temp / t)).exp();
    ReactionRate::new(rate)
}
