/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::dynamics::quantities::Length;
use crate::photonics::quantities::{RayAngle, Wavelength};
use crate::{PhysicsError, PhysicsErrorEnum};
use std::f64::consts::PI;

/// Calculates the Single Slit Diffraction Irradiance.
///
/// Fraunhofer approximation:
/// $$ I(\theta) = I_0 \left( \frac{\sin \beta}{\beta} \right)^2 $$
/// where $\beta = \frac{\pi a \sin \theta}{\lambda}$.
///
/// # Arguments
/// *   `i0` - Peak irradiance $I_0$ at $\theta = 0$.
/// *   `slit_width` - Width of the slit $a$.
/// *   `theta` - Diffraction angle $\theta$.
/// *   `wavelength` - Wavelength $\lambda$.
///
/// # Returns
/// *   `Result<f64, PhysicsError>` - Irradiance at angle $\theta$.
pub fn single_slit_irradiance_kernel(
    i0: f64,
    slit_width: Length,
    theta: RayAngle,
    wavelength: Wavelength,
) -> Result<f64, PhysicsError> {
    if i0 < 0.0 {
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken("Irradiance i0 cannot be negative".into()),
        ));
    }

    let a = slit_width.value();
    let lambda = wavelength.value();
    let angle = theta.value();

    if lambda == 0.0 {
        return Err(PhysicsError::new(PhysicsErrorEnum::Singularity(
            "Wavelength is zero".into(),
        )));
    }

    let beta = (PI * a * angle.sin()) / lambda;

    // Limit lim beta->0 (sin beta / beta) = 1
    if beta.abs() < 1e-9 {
        return Ok(i0);
    }

    let sinc = beta.sin() / beta;
    let i = i0 * sinc * sinc;

    Ok(i)
}

/// Calculates the diffraction angle for a Grating using the Grating Equation.
///
/// $$ d (\sin \theta_m - \sin \theta_i) = m \lambda $$
///
/// Solves for $\theta_m$.
///
/// # Arguments
/// *   `pitch` - Grating pitch $d$ (distance between grooves).
/// *   `order` - Diffraction order $m$ (integer).
/// *   `incidence` - Angle of incidence $\theta_i$.
/// *   `wavelength` - Wavelength $\lambda$.
///
/// # Returns
/// *   `Result<RayAngle, PhysicsError>` - Diffraction angle $\theta_m$.
pub fn grating_equation_kernel(
    pitch: Length,
    order: i32,
    incidence: RayAngle,
    wavelength: Wavelength,
) -> Result<RayAngle, PhysicsError> {
    let d = pitch.value();
    let m = order as f64;
    let lambda = wavelength.value();
    let theta_i = incidence.value();

    if d <= 0.0 {
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken("Grating pitch must be positive".into()),
        ));
    }

    // sin(theta_m) = (m * lambda / d) + sin(theta_i)
    let sin_theta_m = (m * lambda / d) + theta_i.sin();

    if sin_theta_m.abs() > 1.0 {
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                "Diffraction order {} does not exist for this configuration (sin_theta = {})",
                order, sin_theta_m
            )),
        ));
    }

    let theta_m = sin_theta_m.asin();
    RayAngle::new(theta_m)
}
