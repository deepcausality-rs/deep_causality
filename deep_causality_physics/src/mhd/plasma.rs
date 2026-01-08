/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::constants::BOLTZMANN_CONSTANT;
use crate::mhd::quantities::{DebyeLength, LarmorRadius};
use crate::{Mass, PhysicalField, PhysicsError, Speed, Temperature};
use deep_causality_multivector::MultiVector;

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
/// *   `Result<DebyeLength, PhysicsError>` - Debye length.
pub fn debye_length_kernel(
    temp: Temperature,
    density_n: f64,
    epsilon_0: f64,
    elementary_charge: f64,
) -> Result<DebyeLength, PhysicsError> {
    if density_n <= 0.0 {
        return Err(PhysicsError::Singularity("Density must be positive".into()));
    }
    if epsilon_0 <= 0.0 {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Permittivity must be positive".into(),
        ));
    }

    let num = epsilon_0 * BOLTZMANN_CONSTANT * temp.value();
    let den = density_n * elementary_charge.powi(2);

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
/// *   `Result<LarmorRadius, PhysicsError>` - Larmor radius $r_L$.
pub fn larmor_radius_kernel(
    mass: Mass,
    velocity_perp: Speed,
    charge: f64,
    b_field: &PhysicalField,
) -> Result<LarmorRadius, PhysicsError> {
    let b_mag = b_field.inner().squared_magnitude().sqrt();

    if b_mag == 0.0 {
        return Err(PhysicsError::Singularity(
            "Zero magnetic field leads to infinite Larmor radius".into(),
        ));
    }
    if charge == 0.0 {
        return Err(PhysicsError::Singularity(
            "Zero charge particle moves in straight line (infinite radius)".into(),
        ));
    }

    let num = mass.value() * velocity_perp.value();
    let den = charge.abs() * b_mag;

    LarmorRadius::new(num / den)
}
