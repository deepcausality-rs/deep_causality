/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Forward relativistic clock kernels — the **complement** of [`solve_gm_analytical_kernel`].
//!
//! Where `solve_gm` *inverts* the weak-field clock equation to recover `GM`, these kernels evaluate it
//! *forward*: given a clock's dynamical state they return its proper-time rate, and the offset between two
//! clocks. This is the missing forward `dτ/dt` primitive identified in the Gap-3 trajectory-axis
//! feasibility study (`openspec/notes/plasma-blackout/gap-3/gap-three-resolution-3-trajectory-axis.md`,
//! FS-3), which validated it against the textbook GPS relativistic split (+45.7 / −7.2 / +38.5 µs/day).
//!
//! [`solve_gm_analytical_kernel`]: crate::solve_gm_analytical_kernel

use crate::{PhysicsError, SPEED_OF_LIGHT};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// The fractional proper-time rate offset `dτ/dt − 1` of a clock at radius `r` moving at speed `v`, to
/// first post-Newtonian (1PN) order in a monopole field:
///
/// $$ \frac{d\tau}{dt} - 1 = \frac{\Phi}{c^2} - \frac{v^2}{2c^2}, \qquad \Phi = -\frac{GM}{r}. $$
///
/// The return value is the dimensionless frequency offset relative to coordinate time (the quantity the
/// `SpaceTimeCoordinate::clock_drift_rate` field carries, i.e. `𝒯 − 1`). It is negative deep in a
/// gravitational well and for fast motion (both slow the clock); positive at higher potential.
///
/// # Arguments
/// * `radius` — distance `r` from the central body's centre of mass (m, > 0).
/// * `speed` — inertial-frame speed `v` (m/s, ≥ 0). **Must** be measured in a non-rotating frame (ECI for
///   Earth); a body-fixed velocity silently omits the Sagnac term.
/// * `gravitational_parameter` — `GM` of the central body (m³/s², > 0).
///
/// # Assumptions
/// Weak field (`|Φ|/c² ≪ 1`) and slow motion (`v²/c² ≪ 1`) — both ~1e-10 at Earth/GNSS scale. Monopole
/// potential: the `J2` quadrupole and higher harmonics are omitted (their clock contribution is below the
/// 1PN terms by orders of magnitude; FS-3 reproduces the GPS split to sub-µs/day without them).
///
/// # Errors
/// [`PhysicsError::Singularity`] / [`PhysicsError::PhysicalInvariantBroken`] on non-positive `radius`/`GM`
/// or negative `speed`; numeric-conversion failures.
///
/// # References
/// * Ashby, N., "Relativity in the Global Positioning System," *Living Reviews in Relativity* 6, 1 (2003).
/// * IERS Conventions (2010), IERS Technical Note 36 — relativistic time scales.
pub fn relativistic_clock_drift_rate_kernel<R>(
    radius: R,
    speed: R,
    gravitational_parameter: R,
) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if radius <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Radius must be positive for the relativistic clock rate".into(),
        ));
    }
    if speed < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Speed must be non-negative".into(),
        ));
    }
    if gravitational_parameter <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Gravitational parameter GM must be positive".into(),
        ));
    }
    let c = R::from_f64(SPEED_OF_LIGHT)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(SPEED_OF_LIGHT)".into()))?;
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0)".into()))?;
    let c2 = c * c;
    // Φ/c² − v²/(2c²), Φ = −GM/r.
    let gravitational = -(gravitational_parameter / radius) / c2;
    let kinematic = -(speed * speed) / (two * c2);
    Ok(gravitational + kinematic)
}

/// The fractional clock-rate **offset** of a moving clock relative to a reference clock — both in the same
/// monopole field — to 1PN order: `[Φ_clock − Φ_ref]/c² − [v_clock² − v_ref²]/(2c²)`. The leading `1`s
/// cancel, so this is the directly-measurable frequency difference (e.g. a GPS satellite clock vs a geoid
/// clock). Multiply by elapsed coordinate time to get the accumulated `τ` offset.
///
/// # Arguments
/// * `radius_clock`, `speed_clock` — the moving clock's radius (m) and inertial speed (m/s).
/// * `radius_reference`, `speed_reference` — the reference clock's radius and speed (e.g. `R_⊕`, 0 for a
///   non-rotating geoid clock).
/// * `gravitational_parameter` — `GM` (m³/s²).
///
/// # Errors
/// Propagates [`relativistic_clock_drift_rate_kernel`] for either clock.
///
/// # References
/// * Ashby, N., *Living Reviews in Relativity* 6, 1 (2003).
pub fn relativistic_clock_offset_kernel<R>(
    radius_clock: R,
    speed_clock: R,
    radius_reference: R,
    speed_reference: R,
    gravitational_parameter: R,
) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let rate_clock =
        relativistic_clock_drift_rate_kernel(radius_clock, speed_clock, gravitational_parameter)?;
    let rate_reference = relativistic_clock_drift_rate_kernel(
        radius_reference,
        speed_reference,
        gravitational_parameter,
    )?;
    Ok(rate_clock - rate_reference)
}
