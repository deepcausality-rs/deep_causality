/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use crate::kernels::dynamics::quantities::Length;
use crate::kernels::photonics::quantities::{RayAngle, Wavelength};
use deep_causality_num::{FromPrimitive, RealField};

/// Calculates the Single Slit Diffraction Irradiance.
///
/// Fraunhofer approximation:
/// $$ I(\theta) = I_0 \left( \frac{\sin \beta}{\beta} \right)^2 $$
/// where $\beta = \frac{\pi a \sin \theta}{\lambda}$.
pub fn single_slit_irradiance_kernel<R>(
    i0: R,
    slit_width: Length<R>,
    theta: RayAngle<R>,
    wavelength: Wavelength<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if i0 < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Irradiance i0 cannot be negative".into(),
        ));
    }

    let a = slit_width.value();
    let lambda = wavelength.value();
    let angle = theta.value();

    if lambda == R::zero() {
        return Err(PhysicsError::Singularity("Wavelength is zero".into()));
    }

    // Use R::pi() — the precision-native π — instead of round-tripping the f64
    // literal through R::from_f64, which would cap precision at f64 even for
    // Float106 callers.
    let beta = (R::pi() * a * angle.sin()) / lambda;

    // Precision-aware near-zero check on beta: at f32, 1e-9 is well below
    // representable precision; at Float106 it is far too loose. `sqrt(epsilon)`
    // is the natural first-order tolerance for "is this number near zero".
    let eps = R::epsilon().sqrt();
    if beta.abs() < eps {
        return Ok(i0);
    }

    let sinc = beta.sin() / beta;
    let i = i0 * sinc * sinc;

    Ok(i)
}

/// Calculates the diffraction angle for a Grating using the Grating Equation.
///
/// $$ d (\sin \theta_m - \sin \theta_i) = m \lambda $$
pub fn grating_equation_kernel<R>(
    pitch: Length<R>,
    order: i32,
    incidence: RayAngle<R>,
    wavelength: Wavelength<R>,
) -> Result<RayAngle<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let d = pitch.value();
    let m = R::from_f64(order as f64)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(order)".into()))?;
    let lambda = wavelength.value();
    let theta_i = incidence.value();

    if d <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Grating pitch must be positive".into(),
        ));
    }

    // sin(theta_m) = (m * lambda / d) + sin(theta_i)
    let sin_theta_m = (m * lambda / d) + theta_i.sin();

    if sin_theta_m.abs() > R::one() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Diffraction order does not exist for this configuration".into(),
        ));
    }

    let theta_m = sin_theta_m.asin();
    RayAngle::<R>::new(theta_m)
}
