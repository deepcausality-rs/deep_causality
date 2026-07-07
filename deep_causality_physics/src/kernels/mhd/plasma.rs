/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::constants::{
    BOLTZMANN_CONSTANT, ELECTRON_MASS, ELEMENTARY_CHARGE, VACUUM_ELECTRIC_PERMITTIVITY,
};
use crate::{DebyeLength, ElectronDensity, LarmorRadius, PlasmaFrequency};
use crate::{Mass, PhysicalField, PhysicsError, Speed, Temperature};
use deep_causality_algebra::RealField;
use deep_causality_multivector::MultiVector;
use deep_causality_num::FromPrimitive;

/// Calculates the Debye Length $\lambda_D$.
/// $$ \lambda_D = \sqrt{\frac{\epsilon_0 k_B T_e}{n_e e^2}} $$
///
/// # Arguments
/// *   `temp` - Electron temperature $T_e$.
/// *   `density_n` - Electron number density $n_e$ ($m^{-3}$).
/// *   `epsilon_0` - Permittivity of free space.
/// *   `elementary_charge` - Charge $e$.
///
/// # Returns
/// *   `Result<DebyeLength<R>, PhysicsError>` - Debye length.
pub fn debye_length_kernel<R>(
    temp: Temperature<R>,
    density_n: R,
    epsilon_0: R,
    elementary_charge: R,
) -> Result<DebyeLength<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if density_n <= R::zero() {
        return Err(PhysicsError::Singularity("Density must be positive".into()));
    }
    if epsilon_0 <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Permittivity must be positive".into(),
        ));
    }

    let kb = R::from_f64(BOLTZMANN_CONSTANT).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(BOLTZMANN_CONSTANT) failed".into())
    })?;

    let num = epsilon_0 * kb * temp.value();
    let den = density_n * (elementary_charge * elementary_charge);

    let lambda = (num / den).sqrt();
    DebyeLength::new(lambda)
}

/// Calculates the Larmor Radius (Gyroradius).
/// $$ r_L = \frac{m v_\perp}{|q| B} $$
///
/// # Arguments
/// *   `mass` - Particle mass $m$.
/// *   `velocity_perp` - Perpendicular velocity $v_\perp$.
/// *   `charge` - Particle charge $q$.
/// *   `b_field` - Magnetic field $B$.
///
/// # Returns
/// *   `Result<LarmorRadius<R>, PhysicsError>` - Larmor radius $r_L$.
pub fn larmor_radius_kernel<R>(
    mass: Mass<R>,
    velocity_perp: Speed<R>,
    charge: R,
    b_field: &PhysicalField<R>,
) -> Result<LarmorRadius<R>, PhysicsError>
where
    R: RealField,
{
    let b_mag = b_field.inner().squared_magnitude().sqrt();

    if b_mag == R::zero() {
        return Err(PhysicsError::Singularity(
            "Zero magnetic field leads to infinite Larmor radius".into(),
        ));
    }
    if charge == R::zero() {
        return Err(PhysicsError::Singularity(
            "Zero charge particle moves in straight line (infinite radius)".into(),
        ));
    }

    let num = mass.value() * velocity_perp.value();
    let den = charge.abs() * b_mag;

    LarmorRadius::new(num / den)
}

/// Calculates the (angular) electron plasma frequency $\omega_p$.
/// $$ \omega_p = \sqrt{\frac{n_e e^2}{\epsilon_0 m_e}} $$
///
/// Constructs the existing `PlasmaFrequency` newtype, reusing the universal
/// constants `ELECTRON_MASS`, `VACUUM_ELECTRIC_PERMITTIVITY`, `ELEMENTARY_CHARGE`.
/// The blackout criterion compares this against the comms-band angular frequency.
///
/// # Arguments
/// * `electron_density` - Electron number density $n_e$ ($m^{-3}$).
///
/// # Returns
/// * `Result<PlasmaFrequency<R>, PhysicsError>` - Angular plasma frequency (rad/s).
pub fn plasma_frequency_kernel<R>(
    electron_density: ElectronDensity<R>,
) -> Result<PlasmaFrequency<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let n_e = electron_density.value();

    let eps0 = R::from_f64(VACUUM_ELECTRIC_PERMITTIVITY).ok_or_else(|| {
        PhysicsError::NumericalInstability(
            "R::from_f64(VACUUM_ELECTRIC_PERMITTIVITY) failed".into(),
        )
    })?;
    let m_e = R::from_f64(ELECTRON_MASS).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(ELECTRON_MASS) failed".into())
    })?;
    let e = R::from_f64(ELEMENTARY_CHARGE).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(ELEMENTARY_CHARGE) failed".into())
    })?;

    let omega_p = (n_e * e * e / (eps0 * m_e)).sqrt();
    PlasmaFrequency::new(omega_p)
}
