/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::universal::{G, SPEED_OF_LIGHT};
use crate::dynamics::quantities::{Length, Mass, Speed};
use crate::error::PhysicsError;
use deep_causality_core::{CausalityError, PropagatingEffect};

// Kernels

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

pub fn schwarzschild_radius_kernel(mass: &Mass) -> Result<Length, PhysicsError> {
    // r_s = 2GM / c^2
    let c = SPEED_OF_LIGHT;
    let num = 2.0 * G * mass.value();
    let den = c * c;
    let r = num / den;
    Length::new(r)
}

// Wrappers

pub fn orbital_velocity(mass: &Mass, radius: &Length) -> PropagatingEffect<Speed> {
    match orbital_velocity_kernel(mass, radius) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn escape_velocity(mass: &Mass, radius: &Length) -> PropagatingEffect<Speed> {
    match escape_velocity_kernel(mass, radius) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn schwarzschild_radius(mass: &Mass) -> PropagatingEffect<Length> {
    match schwarzschild_radius_kernel(mass) {
        Ok(r) => PropagatingEffect::pure(r),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
