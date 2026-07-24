/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The cohesive GNSS-denial navigation engine (Gap-3 B1+B2+B3) — the trajectory/nav engine that ties
//! together the shipped primitives:
//!
//! * **B1 nominal** — the [`KsPropagator`](crate::KsPropagator) conformal core with the between-step aero
//!   kick ([`ks_strang_step`](crate::ks_strang_step)); the Cartesian nominal is re-lifted to the KS
//!   manifold each step, so it stays a valid bound orbit.
//! * **B2 filter** — the 17-state error-state [`NavFilter`](super::NavFilter); a measurement update
//!   corrects the nominal (inject) and the KS re-lift is the projection back onto the constraint manifold
//!   (a corrected state that is still a bound Kepler orbit satisfies the KS bilinear gauge by
//!   construction).
//! * **B3 clock** — the shipped [`relativistic_clock_drift_rate_kernel`](crate::relativistic_clock_drift_rate_kernel)
//!   carried on **proper time `τ`**, kept distinct from the KS fictitious time `s` the propagator advances
//!   on internally (the FS-3 correction: `s ≠ τ`).
//!
//! The load-bearing behaviour: through a blackout the engine dead-reckons (predict-only, uncertainty
//! grows, clock carried); on GNSS/optical reacquisition a position fix collapses the error and the
//! trajectory stays on the valid-orbit manifold.

use super::eskf::NavFilter;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Quaternion;
use deep_causality_physics::{
    KsPropagator, PhysicsError, ks_strang_step, relativistic_clock_drift_rate_kernel,
};

/// The onboard reentry trajectory + navigation engine.
#[derive(Clone, Debug)]
pub struct ReentryNavEngine<R: RealField + FromPrimitive> {
    gm: R,
    position: [R; 3],
    velocity: [R; 3],
    filter: NavFilter<R>,
    /// The nominal **body→nav attitude**. Integrated from the sensed angular rate each predict and
    /// corrected by the estimated `δψ` on each fix, this is what makes the ESKF's attitude-error state an
    /// error *about a nominal* rather than an estimate with nowhere to go: the reset in
    /// [`correct_position`](Self::correct_position) may zero the attitude block precisely because the
    /// correction was injected here first. Its DCM `attitude.to_rotation_matrix()` rotates the body-frame
    /// specific force into the nav frame the filter's `−[f]×` coupling reads; the identity quaternion
    /// reproduces the Tier-A `C ≈ I` model exactly (a non-rotating vehicle stays at identity, so the
    /// numbers are unchanged).
    attitude: Quaternion<R>,
    /// Carried proper-time offset `τ − t` (s) — the relativistic clock correction, **not** the KS
    /// fictitious time `s`.
    tau_offset: R,
    elapsed: R,
}

impl<R: RealField + FromPrimitive> ReentryNavEngine<R> {
    /// Build the engine from an initial Cartesian nominal `(position, velocity)`, the primary
    /// gravitational parameter `gm`, and an initialised error-state filter. The nominal attitude starts
    /// at identity (`C ≈ I`); a known initial attitude can be injected afterwards via a fix, or the
    /// engine can be [`restore`](Self::restore)d with one.
    pub fn new(position: [R; 3], velocity: [R; 3], gm: R, filter: NavFilter<R>) -> Self {
        Self {
            gm,
            position,
            velocity,
            filter,
            attitude: Quaternion::identity(),
            tau_offset: R::zero(),
            elapsed: R::zero(),
        }
    }

    /// One predict step of `dt`: integrate the nominal attitude from the sensed body angular rate,
    /// advance the nominal (KS drift + the ④ aero acceleration as a Strang kick), propagate the
    /// error-state covariance, and accumulate the carried relativistic clock offset `τ` from the real
    /// orbit geometry. `process_noise` is the ESKF `Q` continuous-time spectral density (see
    /// [`NavFilter::predict`]).
    ///
    /// `angular_rate` is the **sensed body angular rate `ω̂`** (rad/s) — the gyro's measurement, true
    /// rate plus bias. It integrates the nominal body→nav attitude; the resulting DCM `C(q)` rotates the
    /// body-frame specific force `aero_accel` into the nav frame the filter's `−[f]×` coupling reads,
    /// replacing the Tier-A `C ≈ I` assumption. For a non-rotating vehicle (`ω̂ = 0`, the point-mass
    /// examples) the attitude stays exactly identity and `C(q)·f = f`, so the numbers are unchanged.
    ///
    /// # Errors
    /// Propagates KS-propagation / clock-kernel failures (the half-kicked state must stay bound).
    pub fn predict(
        &mut self,
        dt: R,
        aero_accel: [R; 3],
        angular_rate: [R; 3],
        process_noise: [R; 17],
    ) -> Result<(), PhysicsError> {
        // Integrate the nominal attitude: q ← normalize(q ⊗ Δq), Δq = from_axis_angle(ω̂, |ω̂|·dt). A zero
        // rate yields the identity rotor (from_axis_angle returns identity for a zero-length axis), so a
        // non-rotating vehicle leaves the nominal exactly at identity.
        let delta = Quaternion::from_axis_angle(angular_rate, norm(angular_rate) * dt);
        self.attitude = (self.attitude * delta).normalize();

        let (r1, v1) = ks_strang_step(self.position, self.velocity, self.gm, dt, |_r, _v| {
            aero_accel
        })?;
        self.position = r1;
        self.velocity = v1;
        // The accelerometer senses the non-gravitational (aero) specific force in the *body* frame;
        // rotate it into the nav frame via the nominal DCM before the filter's error-dynamics use it.
        let f_nav = mat3_vec(&self.attitude.to_rotation_matrix(), aero_accel);
        self.filter.predict(dt, f_nav, process_noise);
        // Carried clock: dτ/dt − 1 at the current geometry, integrated on proper time (s ≠ τ).
        let radius = norm(r1);
        let speed = norm(v1);
        let rate = relativistic_clock_drift_rate_kernel(radius, speed, self.gm)?;
        self.tau_offset += rate * dt;
        self.elapsed += dt;
        Ok(())
    }

    /// Fold in a position fix (GNSS or through-plasma optical): three sequential scalar updates of the
    /// per-axis position error, then **inject** the estimated error into the nominal and reset the
    /// navigation error (the ESKF feedback). `r_var` is the per-axis measurement variance.
    ///
    /// # Errors
    /// Propagates a refusal from [`NavFilter::update_scalar`] — a degenerate measurement (non-finite or
    /// negative `r_var`, or a zero-variance axis met with a zero-variance fix) — so a caller learns the
    /// fix was **not** folded rather than continuing as if it had been. The three per-axis folds are
    /// applied atomically: the filter is snapshotted first and rolled back on any rejection, so a
    /// refusal on the second axis never leaves the first axis's fold applied (nor a partial correction
    /// injected into the nominal).
    pub fn correct_position(
        &mut self,
        measured_position: [R; 3],
        r_var: R,
    ) -> Result<(), PhysicsError> {
        let backup = self.filter.clone();
        for (i, &m) in measured_position.iter().enumerate() {
            let z = m - self.position[i];
            let mut h = [R::zero(); 17];
            h[i] = R::one();
            if let Err(e) = self.filter.update_scalar(h, z, r_var) {
                self.filter = backup;
                return Err(e);
            }
        }
        let est = *self.filter.state();
        let dp = est.position_error();
        let dv = est.velocity_error();
        let dpsi = est.attitude_error();
        self.position = core::array::from_fn(|i| self.position[i] + dp[i]);
        self.velocity = core::array::from_fn(|i| self.velocity[i] + dv[i]);
        // Inject the estimated attitude error into the nominal: q ← normalize(Δq ⊗ q), where Δq rotates
        // by the small-angle error `δψ`. Only after this is the reset below legitimate — the attitude
        // block is zeroed *because* its estimate was transferred to the nominal, not discarded. For the
        // point-mass examples `δψ ≈ 0`, so this leaves the nominal at identity but the covariance
        // reduction is now justified by an applied (≈ zero) correction rather than claimed for free.
        let correction = Quaternion::from_axis_angle(dpsi, norm(dpsi));
        self.attitude = (correction * self.attitude).normalize();
        self.filter.reset_navigation_error();
        Ok(())
    }

    /// The current Cartesian nominal position.
    pub fn position(&self) -> [R; 3] {
        self.position
    }
    /// The current Cartesian nominal velocity.
    pub fn velocity(&self) -> [R; 3] {
        self.velocity
    }
    /// The carried relativistic clock offset `τ − t` (s) — distinct from the KS fictitious time.
    pub fn carried_clock_offset(&self) -> R {
        self.tau_offset
    }
    /// Elapsed physical (coordinate) time (s).
    pub fn elapsed_time(&self) -> R {
        self.elapsed
    }
    /// The filter's position-error variance — the reacquisition witness.
    pub fn position_variance(&self) -> R {
        self.filter.position_variance()
    }
    /// The primary gravitational parameter the engine propagates against.
    pub fn gm(&self) -> R {
        self.gm
    }

    /// Rebuild an engine from snapshotted state: the exact inverse of reading
    /// [`position`](Self::position), [`velocity`](Self::velocity), [`gm`](Self::gm),
    /// [`filter`](Self::filter), [`attitude`](Self::attitude),
    /// [`carried_clock_offset`](Self::carried_clock_offset), and [`elapsed_time`](Self::elapsed_time).
    /// Exists for the state-snapshot resume path, so a restored engine is bit-identical to the
    /// suspended one — including the nominal attitude, so a resumed rotating vehicle keeps its heading.
    pub fn restore(
        position: [R; 3],
        velocity: [R; 3],
        gm: R,
        filter: NavFilter<R>,
        attitude: Quaternion<R>,
        tau_offset: R,
        elapsed: R,
    ) -> Self {
        Self {
            gm,
            position,
            velocity,
            filter,
            attitude,
            tau_offset,
            elapsed,
        }
    }

    /// The error-state filter.
    pub fn filter(&self) -> &NavFilter<R> {
        &self.filter
    }

    /// The nominal body→nav attitude (the reacquisition/resume witness for the attitude channel).
    pub fn attitude(&self) -> Quaternion<R> {
        self.attitude
    }

    /// Whether the current nominal is a bound Kepler orbit — i.e. it lifts back onto the KS constraint
    /// manifold (the B2 projection invariant: a valid re-lift means the bilinear gauge is satisfied).
    pub fn is_on_orbit_manifold(&self) -> bool {
        KsPropagator::from_state(self.position, self.velocity, self.gm).is_ok()
    }
}

fn norm<R: RealField>(v: [R; 3]) -> R {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Rotate a 3-vector by a 3×3 matrix: `m · v`. Used to carry the body-frame specific force into the nav
/// frame via the nominal attitude's DCM.
fn mat3_vec<R: RealField>(m: &[[R; 3]; 3], v: [R; 3]) -> [R; 3] {
    core::array::from_fn(|i| m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2])
}
