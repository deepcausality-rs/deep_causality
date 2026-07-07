/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Exact two-body (Kepler) propagation as a **constant-generator matrix exponential** — the reusable core
//! validated in the Gap-3 trajectory-axis feasibility study
//! (`openspec/notes/plasma-blackout/gap-3/gap-three-resolution-3-trajectory-axis.md`, FS-1).
//!
//! The bound inverse-square orbit is *exactly linear* under the eccentric-anomaly reparametrisation: in
//! eccentric anomaly `s = E`, the recentred perifocal coordinate `Q = (a·cos E, b·sin E)` solves the
//! unit-frequency harmonic oscillator `Q'' = −Q`, so the phase state `ψ = (Q, Q')` advances by the
//! **constant** symplectic generator `Ω = [[0, I], [−I, 0]]` whose exponential is the cos/sin block
//! `e^{Ω·s}`. Physical time follows from Kepler's equation `M = E − e·sin E`, `t = M/n`. FS-1 measured this
//! reproducing a Kepler orbit to ~1e-15·a — i.e. the "exact conformal core" of the perturbed-conformal
//! trajectory split (Resolution 1, B1) is concrete and textbook; the heavier Bars `(4,2)` packaging is not
//! required.
//!
//! This type is the **planar** realisation (the orbital plane). The 3-D, singularity-free generalisation is
//! Kustaanheimo–Stiefel regularisation (the same constant-generator idea lifted to a 4-D oscillator); it is
//! the documented production extension.
//!
//! # References
//! * Stiefel, E. L. & Scheifele, G., *Linear and Regular Celestial Mechanics*, Springer (1971).
//! * Battin, R. H., *An Introduction to the Mathematics and Methods of Astrodynamics*, AIAA (1999).

use crate::PhysicsError;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// An exact planar two-body (Kepler) propagator built from a single physical state. Propagation is the
/// constant-generator matrix exponential in eccentric anomaly (see the module docs); it is exact to
/// floating-point round-off for any `dt`, with no step-size error.
#[derive(Clone, Copy, Debug)]
pub struct TwoBodyPropagator<R> {
    gravitational_parameter: R,
    semi_major_axis: R,
    eccentricity: R,
    mean_motion: R,
    /// Argument of periapsis in the plane (rad).
    omega_peri: R,
    /// Eccentric anomaly at construction (the epoch state).
    ea0: R,
}

impl<R> TwoBodyPropagator<R>
where
    R: RealField + FromPrimitive,
{
    /// Build the propagator from a planar physical state `(position, velocity)` about a primary with
    /// gravitational parameter `gm = GM`.
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] if `gm ≤ 0` or the state is not a bound ellipse
    /// (`energy ≥ 0`, i.e. `e ≥ 1` — parabolic/hyperbolic are out of scope for this eccentric-anomaly
    /// realisation); [`PhysicsError::Singularity`] on a non-positive radius; numeric-conversion failures.
    pub fn from_state(position: [R; 2], velocity: [R; 2], gm: R) -> Result<Self, PhysicsError> {
        if gm <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Gravitational parameter GM must be positive".into(),
            ));
        }
        let two = Self::lit(2.0)?;
        let one = R::one();

        let r = (position[0] * position[0] + position[1] * position[1]).sqrt();
        if r <= R::zero() {
            return Err(PhysicsError::Singularity("Radius must be positive".into()));
        }
        let v2 = velocity[0] * velocity[0] + velocity[1] * velocity[1];
        let energy = v2 / two - gm / r;
        if energy >= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "State is not a bound orbit (energy >= 0); only ellipses are supported".into(),
            ));
        }
        let semi_major_axis = -gm / (two * energy);
        let rv = position[0] * velocity[0] + position[1] * velocity[1];
        // Eccentricity vector e_vec = ((v² − μ/r)·r − (r·v)·v)/μ.
        let e_vec = [
            ((v2 - gm / r) * position[0] - rv * velocity[0]) / gm,
            ((v2 - gm / r) * position[1] - rv * velocity[1]) / gm,
        ];
        let eccentricity = (e_vec[0] * e_vec[0] + e_vec[1] * e_vec[1]).sqrt();
        if eccentricity >= one {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Non-elliptic orbit (e >= 1) is out of scope".into(),
            ));
        }
        let mean_motion = (gm / (semi_major_axis * semi_major_axis * semi_major_axis)).sqrt();
        let omega_peri = e_vec[1].atan2(e_vec[0]);
        // True anomaly of the epoch point, then eccentric anomaly.
        let cos_nu = Self::clamp_unit(
            (e_vec[0] * position[0] + e_vec[1] * position[1]) / (eccentricity * r),
        );
        let mut nu0 = cos_nu.acos();
        if rv < R::zero() {
            nu0 = two * R::pi() - nu0;
        }
        let ea0 = two
            * ((one - eccentricity).sqrt() * (nu0 / two).sin())
                .atan2((one + eccentricity).sqrt() * (nu0 / two).cos());

        Ok(Self {
            gravitational_parameter: gm,
            semi_major_axis,
            eccentricity,
            mean_motion,
            omega_peri,
            ea0,
        })
    }

    /// The state `(position, velocity)` at time `dt` after the epoch — exact for any `dt`.
    ///
    /// # Errors
    /// Numeric-conversion failures.
    pub fn propagate(&self, dt: R) -> Result<([R; 2], [R; 2]), PhysicsError> {
        let two = Self::lit(2.0)?;
        let one = R::one();
        let e = self.eccentricity;
        let a = self.semi_major_axis;
        let b = a * (one - e * e).sqrt();
        let m0 = self.ea0 - e * self.ea0.sin();
        let ea = Self::solve_kepler(m0 + self.mean_motion * dt, e, two)?;
        // Perifocal position and velocity (the eccentric-anomaly / matrix-exponential advance).
        let x_pf = a * (ea.cos() - e);
        let y_pf = b * ea.sin();
        let edot = self.mean_motion / (one - e * ea.cos());
        let xdot_pf = -a * ea.sin() * edot;
        let ydot_pf = b * ea.cos() * edot;
        // Rotate the perifocal frame into the plane by the argument of periapsis.
        let (c, s) = (self.omega_peri.cos(), self.omega_peri.sin());
        let position = [c * x_pf - s * y_pf, s * x_pf + c * y_pf];
        let velocity = [c * xdot_pf - s * ydot_pf, s * xdot_pf + c * ydot_pf];
        Ok((position, velocity))
    }

    /// Semi-major axis `a` (m).
    pub fn semi_major_axis(&self) -> R {
        self.semi_major_axis
    }
    /// Eccentricity `e` (0 ≤ e < 1).
    pub fn eccentricity(&self) -> R {
        self.eccentricity
    }
    /// Mean motion `n = √(GM/a³)` (rad/s).
    pub fn mean_motion(&self) -> R {
        self.mean_motion
    }
    /// Gravitational parameter `GM` (m³/s²).
    pub fn gravitational_parameter(&self) -> R {
        self.gravitational_parameter
    }
    /// Orbital period `T = 2π/n` (s).
    ///
    /// # Errors
    /// Numeric-conversion failures.
    pub fn period(&self) -> Result<R, PhysicsError> {
        let two = Self::lit(2.0)?;
        Ok(two * R::pi() / self.mean_motion)
    }

    // ── helpers ─────────────────────────────────────────────────────────────────────────────────────

    fn lit(x: f64) -> Result<R, PhysicsError> {
        R::from_f64(x)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64 failed".into()))
    }
    fn clamp_unit(x: R) -> R {
        if x > R::one() {
            R::one()
        } else if x < -R::one() {
            -R::one()
        } else {
            x
        }
    }
    /// Newton solve of Kepler's equation `M = E − e·sin E` for the eccentric anomaly `E`.
    fn solve_kepler(m: R, e: R, two: R) -> Result<R, PhysicsError> {
        let two_pi = two * R::pi();
        // Wrap M into [0, 2π) via floor (no rem_euclid on the generic field).
        let m = m - two_pi * (m / two_pi).floor();
        let tol = Self::lit(1e-15)?;
        let mut ea = m;
        for _ in 0..100 {
            let d = (ea - e * ea.sin() - m) / (R::one() - e * ea.cos());
            ea -= d;
            if d.abs() < tol {
                break;
            }
        }
        Ok(ea)
    }
}
