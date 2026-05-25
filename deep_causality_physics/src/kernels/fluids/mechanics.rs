/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Density, G, Length, PhysicsError, Pressure, Speed};
use deep_causality_num::{FromPrimitive, RealField};

/// Calculates hydrostatic pressure: $P = P_0 + \rho g h$.
///
/// Returns an error when the computed total pressure is negative — the
/// [`Pressure<R>`] newtype rejects sub-vacuum configurations (`P < 0`).
/// This guards against physically unrealisable inputs (e.g. a numerical
/// experiment with a negative reference pressure `P_0` and small `\rho g h`)
/// rather than producing a silently invalid pressure value.
pub fn hydrostatic_pressure_kernel<R>(
    p0: &Pressure<R>,
    density: &Density<R>,
    depth: &Length<R>,
) -> Result<Pressure<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let g = R::from_f64(G)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(G) failed".into()))?;
    let rho_g_h = density.value() * g * depth.value();
    let p_total = p0.value() + rho_g_h;

    Pressure::new(p_total)
}

/// Calculates pressure $P_2$ using Bernoulli's principle.
///
/// $P_1 + \frac{1}{2}\rho v_1^2 + \rho g h_1 = P_2 + \frac{1}{2}\rho v_2^2 + \rho g h_2$
///
/// Solves for $P_2$:
/// $P_2 = P_1 + \frac{1}{2}\rho(v_1^2 - v_2^2) + \rho g(h_1 - h_2)$
pub fn bernoulli_pressure_kernel<R>(
    p1: &Pressure<R>,
    v1: &Speed<R>,
    h1: &Length<R>,
    v2: &Speed<R>,
    h2: &Length<R>,
    density: &Density<R>,
) -> Result<Pressure<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let g = R::from_f64(G)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(G) failed".into()))?;

    let v1_r = v1.value();
    let v2_r = v2.value();
    let h1_r = h1.value();
    let h2_r = h2.value();

    let rho = density.value();
    let term_kinetic = half * rho * (v1_r * v1_r - v2_r * v2_r);
    let term_potential = rho * g * (h1_r - h2_r);

    let p2 = p1.value() + term_kinetic + term_potential;

    Pressure::new(p2)
}
