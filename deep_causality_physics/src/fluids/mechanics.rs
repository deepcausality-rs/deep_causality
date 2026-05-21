/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Density, G, Length, PhysicsError, Pressure, Speed};
use deep_causality_num::{FromPrimitive, RealField};

/// Calculates hydrostatic pressure: $P = P_0 + \rho g h$.
///
/// # Arguments
/// * `p0` - Surface pressure / known reference pressure.
/// * `density` - Fluid density ($\rho$).
/// * `depth` - Depth below the surface or reference point ($h$).
///
/// # Returns
/// * `Ok(Pressure)` - Total pressure at depth.
pub fn hydrostatic_pressure_kernel<R>(
    p0: &Pressure<R>,
    density: &Density<R>,
    depth: &Length,
) -> Result<Pressure<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let g = R::from_f64(G)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(G) failed".into()))?;
    let h = R::from_f64(depth.value())
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(depth) failed".into()))?;
    let rho_g_h = density.value() * g * h;
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
pub fn bernoulli_pressure_kernel<R>(
    p1: &Pressure<R>,
    v1: &Speed,
    h1: &Length,
    v2: &Speed,
    h2: &Length,
    density: &Density<R>,
) -> Result<Pressure<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let lift = |x: f64| -> Result<R, PhysicsError> {
        R::from_f64(x).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64 lift failed in Bernoulli".into())
        })
    };

    let half = lift(0.5)?;
    let g = lift(G)?;
    let v1_r = lift(v1.value())?;
    let v2_r = lift(v2.value())?;
    let h1_r = lift(h1.value())?;
    let h2_r = lift(h2.value())?;

    let rho = density.value();
    let term_kinetic = half * rho * (v1_r * v1_r - v2_r * v2_r);
    let term_potential = rho * g * (h1_r - h2_r);

    let p2 = p1.value() + term_kinetic + term_potential;

    Pressure::new(p2)
}
