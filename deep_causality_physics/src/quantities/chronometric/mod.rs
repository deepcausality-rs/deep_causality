/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{EARTH_GM, EARTH_J2, EARTH_RADIUS_EQUATORIAL, SPEED_OF_LIGHT};
use core::ops::Div;
use deep_causality_algebra::Real;
use deep_causality_num::FromPrimitive;

/// Parameters describing a central gravitating body for weak-field GM recovery.
///
/// J2 (the dimensionless quadrupole moment) is only meaningful relative to a
/// reference equatorial radius — the two travel together by construction, so
/// they live in a single struct rather than scattered top-level constants.
///
/// `gm` is included for forward modeling and reference, even though
/// `solve_gm_analytical_kernel` is what solves for this value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CentralBody<R: Real + Div<Output = R>> {
    /// Body-centric gravitational parameter (m³/s²).
    pub gm: R,
    /// Equatorial reference radius (m) — the radius J2 is referenced to.
    pub equatorial_radius_m: R,
    /// Dimensionless J2 oblateness coefficient.
    pub j2: R,
}

impl CentralBody<f64> {
    /// Earth, JGM-3 J2 with WGS-84 equatorial radius.
    pub const EARTH_JGM3: Self = Self {
        gm: EARTH_GM,
        equatorial_radius_m: EARTH_RADIUS_EQUATORIAL,
        j2: EARTH_J2,
    };
}

impl<R: Real + Div<Output = R>> CentralBody<R> {
    #[inline]
    pub fn new(gm: R, equatorial_radius_m: R, j2: R) -> Self {
        Self {
            gm,
            equatorial_radius_m,
            j2,
        }
    }
}

/// Represents a point in 4D Space-Time with associated kinematic and clock data.
///
/// This is the fundamental that maps a measurable "Clock Event"
/// (timestamp + bias) to a "Geometric Event" (position + velocity) ensuring that
/// $M \leftrightarrow T$ (Mass-Time Equivalence) can be calculated.
///
/// # Type Parameter
///
/// - `R`: real analytic scalar (e.g., `f64`, `Float106`, or `Dual` for forward-mode
///   automatic differentiation) — `Real + Div`, not the stronger `RealField`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpaceTimeCoordinate<R: Real + Div<Output = R>> {
    /// The UTC timestamp in seconds (Unix Epoch).
    pub timestamp: u64,
    /// Satellite ID (e.g., E14)
    pub sat_id: u32,
    /// Radius from Earth's center of mass (meters) $|r|$
    pub r_m: R,
    /// Velocity magnitude relative to Earth (m/s) $|v|$
    pub v_ms: R,
    /// Raw clock bias (seconds), uncorrected for relativistic effects.
    pub clock_bias_s: R,
    /// Full 3D position vector [x, y, z] (ITRF frame)
    pub position: [R; 3],
    /// Full 3D velocity vector [vx, vy, vz] (ITRF frame)
    pub velocity: [R; 3],
    /// Rate of change of clock bias (seconds/second), i.e., frequency offset.
    /// This corresponds to $\mathcal{T} - 1$.
    pub clock_drift_rate: R,
}

impl<R: Real + Div<Output = R> + FromPrimitive> SpaceTimeCoordinate<R> {
    /// Helper to restore relativistic effects removed by IGS.
    /// Calculates $\Delta t_{periodic} = -2(\vec{r} \cdot \vec{v}) / c^2$
    pub fn get_total_bias(&self) -> R {
        let dot_rv = self.position[0] * self.velocity[0]
            + self.position[1] * self.velocity[1]
            + self.position[2] * self.velocity[2];
        let c_sq = R::from_f64(SPEED_OF_LIGHT * SPEED_OF_LIGHT).unwrap();
        let two = R::from_f64(2.0).unwrap();
        let rel_correction = -two * dot_rv / c_sq;
        self.clock_bias_s + rel_correction
    }
}
