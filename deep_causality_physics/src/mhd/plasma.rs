/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::constants::BOLTZMANN_CONSTANT;
use crate::mhd::quantities::{DebyeLength, LarmorRadius};
use crate::{Mass, PhysicalField, PhysicsError, Speed, Temperature};
use deep_causality_multivector::MultiVector;
use deep_causality_num::{FromPrimitive, RealField};

/// Calculates the Debye Length $\lambda_D$.
/// $$ \lambda_D = \sqrt{\frac{\epsilon_0 k_B T_e}{n_e e^2}} $$
///
/// # Arguments
/// *   `temp` - Electron temperature $T_e$ (pinned to `f64` until Temperature is retyped).
/// *   `density_n` - Electron number density $n_e$ ($m^{-3}$).
/// *   `epsilon_0` - Permittivity of free space.
/// *   `elementary_charge` - Charge $e$.
///
/// # Returns
/// *   `Result<DebyeLength<R>, PhysicsError>` - Debye length.
pub fn debye_length_kernel<R>(
    temp: Temperature<f64>,
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
    let temp_r = R::from_f64(temp.value()).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(temp.value()) failed".into())
    })?;

    let num = epsilon_0 * kb * temp_r;
    let den = density_n * (elementary_charge * elementary_charge);

    let lambda = (num / den).sqrt();
    DebyeLength::new(lambda)
}

/// Calculates the Larmor Radius (Gyroradius).
/// $$ r_L = \frac{m v_\perp}{|q| B} $$
///
/// # Arguments
/// *   `mass` - Particle mass $m$ (pinned to `f64` until Mass is retyped).
/// *   `velocity_perp` - Perpendicular velocity $v_\perp$ (pinned to `f64` until Speed is retyped).
/// *   `charge` - Particle charge $q$.
/// *   `b_field` - Magnetic field $B$.
///
/// # Returns
/// *   `Result<LarmorRadius<R>, PhysicsError>` - Larmor radius $r_L$.
pub fn larmor_radius_kernel<R>(
    mass: Mass,
    velocity_perp: Speed,
    charge: R,
    b_field: &PhysicalField<R>,
) -> Result<LarmorRadius<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
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

    let mass_r = R::from_f64(mass.value()).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(mass.value()) failed".into())
    })?;
    let velocity_r = R::from_f64(velocity_perp.value()).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(velocity_perp.value()) failed".into())
    })?;

    let num = mass_r * velocity_r;
    let den = charge.abs() * b_mag;

    LarmorRadius::new(num / den)
}
