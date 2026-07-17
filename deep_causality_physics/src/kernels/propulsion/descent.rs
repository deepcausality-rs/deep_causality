/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Powered-descent kinematics kernels: the closed-form stopping distance,
//! the ignition-altitude solution, and the suicide-burn deceleration command.
//! These are the Tier-A terminal-guidance closed forms of the retropulsion
//! descent; Apollo polynomial guidance (Klumpp 1974) and convex
//! powered-descent guidance (Açıkmeşe & Ploen 2007) are the named upgrade
//! path beyond them.

use crate::{Acceleration, Length, PhysicsError, Speed};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Stopping distance under constant net deceleration
///
/// $$ d = \frac{v^2}{2\,a_{net}} $$
///
/// # Arguments
/// * `speed` — current speed `v` (m/s, ≥ 0).
/// * `net_deceleration` — constant net deceleration `a_net` (m/s², > 0). A
///   vehicle with thrust-to-weight at or below one has `a_net ≤ 0` and cannot
///   stop; that input is rejected, not extrapolated.
///
/// # References
/// * Closed-form constant-acceleration kinematics; Klumpp, A. R., "Apollo
///   Lunar Descent Guidance," Automatica 10(2), 1974, and Açıkmeşe, B., &
///   Ploen, S. R., JGCD 30(5), 2007, as the guidance upgrade path.
pub fn stopping_distance_kernel<R>(
    speed: Speed<R>,
    net_deceleration: Acceleration<R>,
) -> Result<Length<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let a = net_deceleration.value();
    if a <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Net deceleration must be positive; thrust-to-weight <= 1 cannot stop".into(),
        ));
    }
    let v = speed.value();
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    Length::new(v * v / (two * a))
}

/// Ignition altitude for a constant-thrust vertical stopping burn
///
/// $$ h_{ign} = \frac{v^2}{2\,(a_T - g)} + h_{margin} $$
///
/// the stopping distance against the net deceleration `a_T − g`, plus the
/// caller-supplied margin. The margin is an input by design: downstream it is
/// sized from the weather-dispersion table's navigation-drift row, which is
/// not this crate's business.
///
/// # Arguments
/// * `speed` — descent speed at ignition `v` (m/s, ≥ 0).
/// * `thrust_acceleration` — thrust acceleration `a_T = T/m` (m/s², must
///   exceed `gravity`).
/// * `gravity` — local gravitational acceleration `g` (m/s², > 0).
/// * `margin` — additive altitude margin `h_margin` (m, ≥ 0).
///
/// # References
/// * Closed-form constant-acceleration kinematics (see
///   [`stopping_distance_kernel`]).
pub fn ignition_altitude_kernel<R>(
    speed: Speed<R>,
    thrust_acceleration: Acceleration<R>,
    gravity: Acceleration<R>,
    margin: Length<R>,
) -> Result<Length<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let g = gravity.value();
    if g <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Gravitational acceleration must be positive".into(),
        ));
    }
    let a_net = thrust_acceleration.value() - g;
    if a_net <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Thrust acceleration must exceed gravity; thrust-to-weight <= 1 cannot stop".into(),
        ));
    }
    let d = stopping_distance_kernel(speed, Acceleration::new(a_net)?)?;
    Length::new(d.value() + margin.value())
}

/// Suicide-burn deceleration command
///
/// $$ a_{cmd} = \frac{v^2}{2h} + g $$
///
/// the constant deceleration that nulls the descent speed exactly at the
/// surface from the current speed `v` and altitude `h` — the closed-form
/// feedback the terminal-guidance stage clamps into the safety envelope
/// downstream.
///
/// # Arguments
/// * `speed` — current descent speed `v` (m/s, ≥ 0).
/// * `altitude` — current altitude above the surface `h` (m, > 0). Ground
///   contact (`h ≤ 0`) is rejected.
/// * `gravity` — local gravitational acceleration `g` (m/s², > 0).
///
/// # References
/// * Closed-form constant-acceleration kinematics (see
///   [`stopping_distance_kernel`]).
pub fn suicide_burn_deceleration_kernel<R>(
    speed: Speed<R>,
    altitude: Length<R>,
    gravity: Acceleration<R>,
) -> Result<Acceleration<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let h = altitude.value();
    if h <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Altitude must be positive; the vehicle is at or below ground contact".into(),
        ));
    }
    let g = gravity.value();
    if g <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Gravitational acceleration must be positive".into(),
        ));
    }
    let v = speed.value();
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    Acceleration::new(v * v / (two * h) + g)
}
