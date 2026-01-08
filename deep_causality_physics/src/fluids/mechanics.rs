/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Density, G, Length, PhysicsError, Pressure, Speed};

/// Calculates hydrostatic pressure: $P = P_0 + \rho g h$.
///
/// # Arguments
/// * `p0` - Surface pressure / known reference pressure.
/// * `density` - Fluid density ($\rho$).
/// * `depth` - Depth below the surface or reference point ($h$).
///
/// # Returns
/// * `Ok(Pressure)` - Total pressure at depth.
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

/// Calculates pressure $P_2$ using Bernoulli's principle.
///
/// $P_1 + \frac{1}{2}\rho v_1^2 + \rho g h_1 = P_2 + \frac{1}{2}\rho v_2^2 + \rho g h_2$
///
/// Solves for $P_2$:
/// $P_2 = P_1 + \frac{1}{2}\rho(v_1^2 - v_2^2) + \rho g(h_1 - h_2)$
///
/// # Arguments
/// * `p1` - Pressure at point 1.
/// * `v1` - Velocity at point 1.
/// * `h1` - Elevation at point 1.
/// * `v2` - Velocity at point 2.
/// * `h2` - Elevation at point 2.
/// * `density` - Fluid density.
///
/// # Returns
/// * `Ok(Pressure)` - Pressure at point 2.
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
    // Use proper f64 methods
    let term_kinetic = 0.5 * rho * (v1.value().powi(2) - v2.value().powi(2));
    let term_potential = rho * G * (h1.value() - h2.value());

    let p2 = p1.value() + term_kinetic + term_potential;

    Pressure::new(p2)
}
