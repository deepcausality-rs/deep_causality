/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Ideal-flow primitive kernels: dynamic pressure, Bernoulli total head,
//! 2D stream function and velocity potential differential updates, line
//! circulation, and the Kutta–Joukowski lift per unit span.
//!
//! Conventions: standard surface gravity `g = 9.80665 m/s²` (constant `G`)
//! is used for the Bernoulli head. 2D differential updates `dψ`, `dφ` are
//! returned per call; integrating along a path is the caller's concern.

use crate::PhysicsError;
use crate::{Density, G};
use crate::{Length, Speed};
use crate::{Pressure, Velocity3};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Dynamic pressure `q = 0.5 · ρ · u²` (Pa).
///
/// Always non-negative since `ρ ≥ 0` (`Density` invariant) and `u² ≥ 0`.
pub fn dynamic_pressure_kernel<R>(
    rho: &Density<R>,
    u: &Speed<R>,
) -> Result<Pressure<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let v = u.value();
    Pressure::new(half * rho.value() * v * v)
}

/// Bernoulli total head `H = p / (ρ · g) + u² / (2 · g) + h` (m).
///
/// Errors on `ρ = 0` (division by zero) or when [`Length::new`] rejects a
/// negative head.
pub fn bernoulli_total_head_kernel<R>(
    p: &Pressure<R>,
    rho: &Density<R>,
    u: &Speed<R>,
    h: &Length<R>,
) -> Result<Length<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let r = rho.value();
    if r == R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "bernoulli_total_head_kernel: density is zero".into(),
        ));
    }
    let g = R::from_f64(G)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(G) failed".into()))?;
    let two = R::one() + R::one();
    let v = u.value();
    let head = p.value() / (r * g) + (v * v) / (two * g) + h.value();
    Length::new(head)
}

/// 2D stream-function differential update `dψ = u · dy − v · dx`.
///
/// Caller integrates along a path. Pure scalar arithmetic.
pub fn stream_function_2d_kernel<R>(u: R, v: R, dx: R, dy: R) -> R
where
    R: RealField,
{
    u * dy - v * dx
}

/// 2D velocity-potential differential update `dφ = u · dx + v · dy`.
///
/// Caller integrates along a path. Defined only for irrotational flow.
pub fn velocity_potential_2d_kernel<R>(u: R, v: R, dx: R, dy: R) -> R
where
    R: RealField,
{
    u * dx + v * dy
}

/// Circulation `Γ = ∮ u · dl` as a discrete line integral.
///
/// `velocity_at_loop_points[i]` and `tangents[i]` are paired sample points
/// along the closed loop; `tangents[i]` is the directed edge vector `dl_i`.
/// Errors when the two slices have different lengths.
pub fn circulation_kernel<R>(
    velocity_at_loop_points: &[Velocity3<R>],
    tangents: &[[R; 3]],
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    if velocity_at_loop_points.len() != tangents.len() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "circulation_kernel: velocity and tangent slices must have equal length".into(),
        ));
    }
    let mut gamma = R::zero();
    for (u_pt, dl) in velocity_at_loop_points.iter().zip(tangents.iter()) {
        let u = u_pt.value();
        gamma += u[0] * dl[0] + u[1] * dl[1] + u[2] * dl[2];
    }
    Ok(gamma)
}

/// Kutta–Joukowski lift per unit span `L' = ρ · u_∞ · Γ` (N/m).
///
/// Sign convention: positive `Γ` corresponds to clockwise circulation in
/// the conventional 2D setup (x right, y up), giving positive lift.
pub fn kutta_joukowski_lift_kernel<R>(rho: &Density<R>, u_inf: &Speed<R>, circulation: R) -> R
where
    R: RealField,
{
    rho.value() * u_inf.value() * circulation
}
