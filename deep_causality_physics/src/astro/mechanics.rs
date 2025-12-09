/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::PhysicsError;
use crate::{G, SPEED_OF_LIGHT};
use crate::{Length, Mass, Speed};

/// Calculates orbital velocity: $v = \sqrt{\frac{GM}{r}}$.
///
/// # Arguments
/// * `mass_primary` - Mass of the primary body $M$.
/// * `radius` - Orbital radius $r$.
///
/// # Returns
/// * `Ok(Speed)` - Orbital velocity $v$.
pub fn orbital_velocity_kernel(
    mass_primary: &Mass,
    radius: &Length,
) -> Result<Speed, PhysicsError> {
    // v = sqrt(GM / r)
    let gm = G * mass_primary.value();
    if radius.value() == 0.0 {
        return Err(PhysicsError::new(
            crate::error::PhysicsErrorEnum::MetricSingularity(
                "Zero radius in orbital velocity".into(),
            ),
        ));
    }
    let v = (gm / radius.value()).sqrt();
    Speed::new(v)
}

/// Calculates escape velocity: $v_e = \sqrt{\frac{2GM}{r}}$.
///
/// # Arguments
/// * `mass_primary` - Mass of the primary body $M$.
/// * `radius` - Distance from center of mass $r$.
///
/// # Returns
/// * `Ok(Speed)` - Escape velocity $v_e$.
pub fn escape_velocity_kernel(mass_primary: &Mass, radius: &Length) -> Result<Speed, PhysicsError> {
    // v = sqrt(2GM / r)
    let gm = G * mass_primary.value();
    if radius.value() == 0.0 {
        return Err(PhysicsError::new(
            crate::error::PhysicsErrorEnum::MetricSingularity(
                "Zero radius in escape velocity".into(),
            ),
        ));
    }
    let v = (2.0 * gm / radius.value()).sqrt();
    Speed::new(v)
}

/// Calculates Schwarzschild radius: $r_s = \frac{2GM}{c^2}$.
///
/// # Arguments
/// * `mass` - Mass of the object $M$.
///
/// # Returns
/// * `Ok(Length)` - Schwarzschild radius $r_s$.
pub fn schwarzschild_radius_kernel(mass: &Mass) -> Result<Length, PhysicsError> {
    // r_s = 2GM / c^2
    let c = SPEED_OF_LIGHT;
    let num = 2.0 * G * mass.value();
    let den = c * c;
    let r = num / den;
    Length::new(r)
}
