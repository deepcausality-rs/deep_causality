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
use crate::{KsPropagator, PhysicsError, ks_strang_step, relativistic_clock_drift_rate_kernel};
use deep_causality_num::{FromPrimitive, RealField};

/// The onboard reentry trajectory + navigation engine.
#[derive(Clone, Debug)]
pub struct ReentryNavEngine<R: RealField + FromPrimitive> {
    gm: R,
    position: [R; 3],
    velocity: [R; 3],
    filter: NavFilter<R>,
    /// Carried proper-time offset `τ − t` (s) — the relativistic clock correction, **not** the KS
    /// fictitious time `s`.
    tau_offset: R,
    elapsed: R,
}

impl<R: RealField + FromPrimitive> ReentryNavEngine<R> {
    /// Build the engine from an initial Cartesian nominal `(position, velocity)`, the primary
    /// gravitational parameter `gm`, and an initialised error-state filter.
    pub fn new(position: [R; 3], velocity: [R; 3], gm: R, filter: NavFilter<R>) -> Self {
        Self {
            gm,
            position,
            velocity,
            filter,
            tau_offset: R::zero(),
            elapsed: R::zero(),
        }
    }

    /// One predict step of `dt`: advance the nominal (KS drift + the ④ aero acceleration as a Strang
    /// kick), propagate the error-state covariance, and accumulate the carried relativistic clock offset
    /// `τ` from the real orbit geometry. `process_noise` is the ESKF `Q` diagonal.
    ///
    /// # Errors
    /// Propagates KS-propagation / clock-kernel failures (the half-kicked state must stay bound).
    pub fn predict(
        &mut self,
        dt: R,
        aero_accel: [R; 3],
        process_noise: [R; 17],
    ) -> Result<(), PhysicsError> {
        let (r1, v1) = ks_strang_step(self.position, self.velocity, self.gm, dt, |_r, _v| {
            aero_accel
        })?;
        self.position = r1;
        self.velocity = v1;
        // The accelerometer senses only the non-gravitational (aero) specific force.
        self.filter.predict(dt, aero_accel, process_noise);
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
    pub fn correct_position(&mut self, measured_position: [R; 3], r_var: R) {
        for (i, &m) in measured_position.iter().enumerate() {
            let z = m - self.position[i];
            let mut h = [R::zero(); 17];
            h[i] = R::one();
            self.filter.update_scalar(h, z, r_var);
        }
        let est = *self.filter.state();
        let dp = est.position_error();
        let dv = est.velocity_error();
        self.position = core::array::from_fn(|i| self.position[i] + dp[i]);
        self.velocity = core::array::from_fn(|i| self.velocity[i] + dv[i]);
        self.filter.reset_navigation_error();
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
    /// The error-state filter.
    pub fn filter(&self) -> &NavFilter<R> {
        &self.filter
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
