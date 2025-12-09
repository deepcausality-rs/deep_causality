/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::G;
use crate::dynamics::quantities::{Length, Speed};
use crate::error::PhysicsError;
use crate::fluids::quantities::{Density, Pressure};
use deep_causality_core::{CausalityError, PropagatingEffect};
// Kernels

pub fn hydrostatic_pressure_kernel(
    p0: &Pressure,
    density: &Density,
    depth: &Length,
) -> Result<Pressure, PhysicsError> {
    // P = P0 + rho * g * h
    let rho_g_h = density.value() * G * depth.value();
    let p_total = p0.value() + rho_g_h;

    Pressure::new(p_total)
}

pub fn bernoulli_pressure_kernel(
    p1: &Pressure,
    v1: &Speed,
    h1: &Length,
    v2: &Speed,
    h2: &Length,
    density: &Density,
) -> Result<Pressure, PhysicsError> {
    // P1 + 0.5 * rho * v1^2 + rho * g * h1 = P2 + 0.5 * rho * v2^2 + rho * g * h2
    // Solve for P2:
    // P2 = P1 + 0.5*rho*(v1^2 - v2^2) + rho*g*(h1 - h2)

    let rho = density.value();
    let term_kinetic = 0.5 * rho * (v1.value().powi(2) - v2.value().powi(2));
    let term_potential = rho * G * (h1.value() - h2.value());

    let p2 = p1.value() + term_kinetic + term_potential;

    Pressure::new(p2)
}

// Wrappers

pub fn hydrostatic_pressure(
    p0: &Pressure,
    density: &Density,
    depth: &Length,
) -> PropagatingEffect<Pressure> {
    match hydrostatic_pressure_kernel(p0, density, depth) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn bernoulli_pressure(
    p1: &Pressure,
    v1: &Speed,
    h1: &Length,
    v2: &Speed,
    h2: &Length,
    density: &Density,
) -> PropagatingEffect<Pressure> {
    match bernoulli_pressure_kernel(p1, v1, h1, v2, h2, density) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
