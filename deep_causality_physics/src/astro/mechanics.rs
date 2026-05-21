/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Length, Mass, PhysicsError, Speed};
use crate::{NEWTONIAN_CONSTANT_OF_GRAVITATION, SPEED_OF_LIGHT};
use deep_causality_num::{FromPrimitive, RealField};

/// Calculates orbital velocity: $v = \sqrt{\frac{GM}{r}}$.
pub fn orbital_velocity_kernel<R>(
    mass_primary: &Mass<R>,
    radius: &Length<R>,
) -> Result<Speed<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let r = radius.value();
    if r <= R::zero() {
        return Err(PhysicsError::MetricSingularity(
            "Non-positive radius in orbital velocity".into(),
        ));
    }
    let g = R::from_f64(NEWTONIAN_CONSTANT_OF_GRAVITATION)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(G) failed".into()))?;
    let gm = g * mass_primary.value();
    let v = (gm / r).sqrt();
    Speed::new(v)
}

/// Calculates escape velocity: $v_e = \sqrt{\frac{2GM}{r}}$.
pub fn escape_velocity_kernel<R>(
    mass_primary: &Mass<R>,
    radius: &Length<R>,
) -> Result<Speed<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if radius.value() == R::zero() {
        return Err(PhysicsError::MetricSingularity(
            "Zero radius in escape velocity".into(),
        ));
    }
    let g = R::from_f64(NEWTONIAN_CONSTANT_OF_GRAVITATION)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(G) failed".into()))?;
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    let gm = g * mass_primary.value();
    let v = (two * gm / radius.value()).sqrt();
    Speed::new(v)
}

/// Calculates Schwarzschild radius: $r_s = \frac{2GM}{c^2}$.
pub fn schwarzschild_radius_kernel<R>(mass: &Mass<R>) -> Result<Length<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let g = R::from_f64(NEWTONIAN_CONSTANT_OF_GRAVITATION)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(G) failed".into()))?;
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    let c = R::from_f64(SPEED_OF_LIGHT)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(c) failed".into()))?;
    let num = two * g * mass.value();
    let den = c * c;
    let r = num / den;
    Length::new(r)
}
