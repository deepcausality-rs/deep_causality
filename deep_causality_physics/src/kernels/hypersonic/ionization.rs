/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Ionization kernels: the Saha equilibrium ionization fraction, the Tier-A
//! Park-2T ionization surrogate (the LER relaxation target), and the
//! electron-density reconstruction.

use crate::constants::{
    BOLTZMANN_CONSTANT, ELECTRON_MASS, ELEMENTARY_CHARGE, NO_IONIZATION_ENERGY_EV, PLANCK_CONSTANT,
};
use crate::{ElectronDensity, IonizationFraction, PhysicsError, Temperature};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Saha-equilibrium ionization fraction `α = n_e / n_tot` for a singly-ionized
/// gas, from
///
/// $$ \frac{n_e n_i}{n_n} = g\,\Big(\frac{2\pi m_e k_B T}{h^2}\Big)^{3/2}
///    \exp\!\Big(-\frac{E_{ion}}{k_B T}\Big) \equiv K(T) $$
///
/// With `n_e = n_i = α n_tot` and `n_n = (1-α) n_tot`, this gives
/// `α²/(1-α) = K/n_tot`, solved as `α = (-x + √(x² + 4x))/2`, `x = K/n_tot`.
/// Saha is the *full* ionization equilibrium — pathway-independent — so it
/// already accounts for electron-impact-produced electrons, not only the
/// associative channel.
///
/// # Arguments
/// * `temperature` — temperature `T` (K).
/// * `total_number_density` — total heavy-particle number density `n_tot` (m⁻³).
/// * `ionization_energy_ev` — ionization energy `E_ion` (eV).
/// * `partition_ratio` — statistical-weight factor `g = 2 g_i / g_n`.
///
/// # References
/// * Gupta, Yos, Thompson & Lee, NASA RP-1232 (1990), eq. 5b
///   (`papers/gupta_1990_nasa_rp1232.pdf`).
pub fn saha_ionization_fraction_kernel<R>(
    temperature: Temperature<R>,
    total_number_density: R,
    ionization_energy_ev: R,
    partition_ratio: R,
) -> Result<IonizationFraction<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let t = temperature.value();
    if t <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Temperature must be positive for the Saha equation".into(),
        ));
    }
    if total_number_density <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Total number density must be positive".into(),
        ));
    }
    if ionization_energy_ev <= R::zero() || partition_ratio <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Ionization energy and partition ratio must be positive".into(),
        ));
    }

    let me_ = R::from_f64(ELECTRON_MASS).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(ELECTRON_MASS) failed".into())
    })?;
    let kb = R::from_f64(BOLTZMANN_CONSTANT).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(BOLTZMANN_CONSTANT) failed".into())
    })?;
    let h = R::from_f64(PLANCK_CONSTANT).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(PLANCK_CONSTANT) failed".into())
    })?;
    let e_charge = R::from_f64(ELEMENTARY_CHARGE).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(ELEMENTARY_CHARGE) failed".into())
    })?;
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    let four = R::from_f64(4.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(4.0) failed".into()))?;
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let three_halves = R::from_f64(1.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(1.5) failed".into()))?;

    let two_pi = two * R::pi();
    let e_ion_j = ionization_energy_ev * e_charge;
    let thermal_db = (two_pi * me_ * kb * t / (h * h)).powf(three_halves);
    let k_saha = partition_ratio * thermal_db * (-(e_ion_j / (kb * t))).exp();

    let x = k_saha / total_number_density;
    // α = (−x + √(x² + 4x)) / 2 ∈ (0, 1).
    let alpha = (-x + (x * x + four * x).sqrt()) * half;
    // Guard the [0, 1] invariant against rounding at the endpoints.
    let alpha = if alpha < R::zero() {
        R::zero()
    } else if alpha > R::one() {
        R::one()
    } else {
        alpha
    };
    IonizationFraction::new(alpha)
}

/// Tier-A Park-2T ionization surrogate: the **relaxation target** `α_eq(T, n)`
/// that the LER `IonizationStage` relaxes toward. It is the Saha equilibrium for
/// the dominant NO ionization channel (`E_ion ≈ 9.26 eV`); being the full
/// equilibrium it carries electron-impact electrons as well as NO⁺. The gap
/// between the carried `α` and this target is the nonequilibrium lag.
///
/// # Arguments
/// * `temperature` — temperature `T` (K).
/// * `total_number_density` — total heavy-particle number density `n_tot` (m⁻³).
///
/// # References
/// * Park, J. Thermophys. Heat Transfer 7(3):385 (1993); Aiken, Carter & Boyd,
///   Plasma Sources Sci. Technol. 34 (2025) — RAM-C (~7.6 km/s) is in the mixed
///   associative + electron-impact ionization band.
pub fn park2t_ionization_surrogate_kernel<R>(
    temperature: Temperature<R>,
    total_number_density: R,
) -> Result<IonizationFraction<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let e_ion = R::from_f64(NO_IONIZATION_ENERGY_EV).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(NO_IONIZATION_ENERGY_EV) failed".into())
    })?;
    // Statistical-weight factor 2·g_i/g_n ≈ 2 for the NO/NO⁺ channel (Tier-A).
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    saha_ionization_fraction_kernel(temperature, total_number_density, e_ion, two)
}

/// Electron density `n_e = α · n_tot` from an ionization fraction.
///
/// # Arguments
/// * `alpha` — ionization fraction `α`.
/// * `total_number_density` — total heavy-particle number density `n_tot` (m⁻³).
pub fn electron_density_kernel<R>(
    alpha: IonizationFraction<R>,
    total_number_density: R,
) -> Result<ElectronDensity<R>, PhysicsError>
where
    R: RealField,
{
    if total_number_density < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Total number density cannot be negative".into(),
        ));
    }
    ElectronDensity::new(alpha.value() * total_number_density)
}
