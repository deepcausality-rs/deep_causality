/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The **terminal-descent guidance stage** and its **ignition-corridor commit** (change
//! `add-retropulsion-terminal-descent`, capabilities `throttle-guidance-stage` and
//! `ignition-corridor-commit`).
//!
//! [`ThrottleGuidance`] is the first producer of the throttle channel anywhere in the workspace:
//! before it, `set_throttle_action` was written only by the safety gate echoing its own clamp back
//! and by snapshot resume, so the gate's whole burn block was unreachable in a composed world.
//!
//! **The law** is the Tier-A closed form `a_cmd = v²/2h + g`
//! (`suicide_burn_deceleration_kernel`), mapped to a throttle through the crate's existing linear
//! thrust convention `T = θ·T₁` — inverted here as `θ = m·a_cmd / T₁` and saturated into `[0, 1]`.
//! There is deliberately **no** thrust-from-throttle kernel: the physics crate leaves throttle
//! mapping to the CFD-side stages, and [`RetroThrust`](super::RetroThrust) already realizes the
//! same relation, so the two share one convention rather than each inventing a map. Apollo
//! polynomial guidance (Klumpp 1974) and convex powered-descent guidance (Açıkmeşe & Ploen 2007)
//! are the named upgrade path, not this stage's scope.
//!
//! **Zero from step 0.** The stage writes the channel on *every* step, commanding zero before the
//! corridor commits. The gate reaches its burn axes only when the throttle channel is present, so a
//! stage that wrote the channel only at ignition would leave the propellant floor, the descent-rate
//! bound, and the throttle clamp unenforced on every pre-ignition step. Commanding zero is safe
//! because the thrust and plume stages are strictly inert at θ ≤ 0, so ignition stays an event on
//! the commanded **value** rather than on the channel's existence.
//!
//! **The commit** ([`IgnitionCorridor`]) is a conjunction of four conditions — Mach band, dynamic
//! pressure inside the envelope's ignition window, a post-fix navigation state, and a navigated
//! position uncertainty inside the table-sized margin — evaluated as a **rising edge** and then
//! **latched one-way**. Two properties force that shape:
//! - the throttle channel is an `Option` that is never cleared between steps and is carried by
//!   clone, so a rising edge cannot be recovered from it and the commit must carry its own edge
//!   state;
//! - carrier-internal state resets at every leg boundary while the coupled field is carried across
//!   it, so the latch rides the `"ignition_committed"` **field scalar**.
//!
//! The latch is one-way by design: an ignition corridor is a decision about when to *start*, not a
//! condition to maintain. Re-opening it every step would let a transient navigation dropout
//! extinguish a burn that is already the only survivable option; safety after commit is the
//! envelope's job, which keeps enforcing every step.
//!
//! **Navigation is read through published scalars, never `field.nav()`.** No stage in this crate
//! steers on navigation quality today, and the published `"nav_mode"` / `"nav_position_variance"`
//! scalars exist for exactly this purpose but have no production consumer — the corridor example
//! reaches *past* them into the engine's own accessor, which is the coupling this stage avoids. Two
//! properties are honored rather than assumed: `"nav_mode"` reports aided versus dead reckoning and
//! does **not** distinguish a GNSS fix from a through-plasma optical fix, so a corridor requiring
//! GNSS reacquisition specifically cannot be expressed against it today; and
//! `"nav_position_variance"` is a covariance **trace in m²**, so a margin in metres is compared
//! against its square root.

use super::coupling::{CoupledField, PhysicsStage, StepContext};
use crate::CfdScalar;
use alloc::vec::Vec;
use deep_causality_haft::LogAddEntry;
use deep_causality_physics::{
    Acceleration, Length, PhysicsError, Speed, suicide_burn_deceleration_kernel,
};

/// The field scalar carrying the one-way ignition latch across steps and leg boundaries.
pub const IGNITION_LATCH_FIELD: &str = "ignition_committed";

/// The four-condition ignition corridor a [`ThrottleGuidance`] commits through.
///
/// The `margin` is supplied by the caller rather than derived here: the ignition-altitude kernel
/// takes its margin as an input by design, because downstream it is sized from the weather
/// dispersion table's navigation-drift row (`drift_mean + k·drift_sd`), which is not this crate's
/// business.
#[derive(Debug, Clone, Copy)]
pub struct IgnitionCorridor<R: CfdScalar> {
    /// Lower edge of the ignition Mach band (inclusive).
    pub mach_min: R,
    /// Upper edge of the ignition Mach band (inclusive).
    pub mach_max: R,
    /// Ignition dynamic-pressure window lower bound, Pa (inclusive).
    pub q_min: R,
    /// Ignition dynamic-pressure window upper bound, Pa (inclusive).
    pub q_max: R,
    /// Largest navigated position uncertainty admitting a commit, m (a 1-sigma distance, compared
    /// against the square root of the published covariance trace).
    pub margin_m: R,
}

impl<R: CfdScalar> IgnitionCorridor<R> {
    /// A corridor over the Mach band `[mach_min, mach_max]`, the dynamic-pressure window
    /// `[q_min, q_max]` (Pa), and a navigated-uncertainty `margin_m` (m).
    pub fn new(mach_min: R, mach_max: R, q_min: R, q_max: R, margin_m: R) -> Self {
        Self {
            mach_min,
            mach_max,
            q_min,
            q_max,
            margin_m,
        }
    }
}

/// The sensed values a commit evaluation saw, carried out of the predicate so the log entry can
/// name them.
#[derive(Debug, Clone, Copy)]
struct CommitSense<R: CfdScalar> {
    mach: R,
    q_inf: R,
    aided: bool,
    sigma_m: R,
}

/// The production **terminal-guidance** stage: commands the throttle from the stopping-distance
/// closed form, behind a latched ignition-corridor commit.
///
/// Construct with [`new`](Self::new) and attach the corridor with
/// [`with_corridor`](Self::with_corridor). Without a corridor the stage commands zero forever — it
/// still writes the channel every step, so the envelope stays live, but it never ignites.
///
/// Composes **after** the navigation stage, so it reads the current step's navigation quality, and
/// **before** the cybernetic gate, so the gate clamps the command it wrote. The thrust and plume
/// stages compose earlier, per the force-channel ordering contract, so they realize the throttle the
/// gate clamped on the **previous** step — a one-step actuation lag inherited from the bank channel,
/// which behaves identically.
#[derive(Debug, Clone, Copy)]
pub struct ThrottleGuidance<R: CfdScalar> {
    thrust: R,
    gravity: R,
    corridor: Option<IgnitionCorridor<R>>,
    speed_field: &'static str,
    altitude_field: &'static str,
    mass_field: &'static str,
    mach_field: &'static str,
    q_field: &'static str,
    nav_mode_field: &'static str,
    nav_var_field: &'static str,
}

impl<R: CfdScalar> ThrottleGuidance<R> {
    /// A guidance stage with full-throttle thrust `thrust` (N) and local gravitational acceleration
    /// `gravity` (m/s²). Field names default to what the compressible carrier and the navigation
    /// stage publish.
    pub fn new(thrust: R, gravity: R) -> Self {
        Self {
            thrust,
            gravity,
            corridor: None,
            speed_field: "flight_speed",
            altitude_field: "flight_altitude",
            mass_field: "mass",
            mach_field: "flight_mach",
            q_field: "q_inf",
            nav_mode_field: "nav_mode",
            nav_var_field: "nav_position_variance",
        }
    }

    /// Attach the ignition corridor. Without one the stage commands zero on every step.
    pub fn with_corridor(mut self, corridor: IgnitionCorridor<R>) -> Self {
        self.corridor = Some(corridor);
        self
    }

    /// Rename the sensed dynamic-pressure field, matching a world that configured the gate's
    /// sensing away from the default.
    pub fn with_q_field(mut self, q_field: &'static str) -> Self {
        self.q_field = q_field;
        self
    }

    /// True once the latch scalar carries a positive value.
    fn committed(&self, field: &CoupledField<R>) -> bool {
        field
            .scalar(IGNITION_LATCH_FIELD)
            .and_then(|s| s.first().copied())
            .is_some_and(|v| v > R::zero())
    }

    /// Evaluate the four corridor conditions against this step's published sensing.
    ///
    /// Returns the sensed values on success, `None` when any condition fails or its scalar is
    /// absent — an absent sensor is a condition not met, never a condition waived.
    fn corridor_holds(&self, field: &CoupledField<R>) -> Option<CommitSense<R>> {
        let c = self.corridor?;
        let first = |name: &str| field.scalar(name).and_then(|s| s.first().copied());

        let mach = first(self.mach_field)?;
        if mach < c.mach_min || mach > c.mach_max {
            return None;
        }

        let q_inf = first(self.q_field)?;
        if q_inf < c.q_min || q_inf > c.q_max {
            return None;
        }

        // "Post-fix" is aided-this-step. Note the published mode does not distinguish a GNSS fix
        // from a through-plasma optical fix.
        let aided = first(self.nav_mode_field)? > R::zero();
        if !aided {
            return None;
        }

        // The published variance is a covariance TRACE in m², so a metre-valued margin compares
        // against its square root.
        let variance = first(self.nav_var_field)?;
        if variance < R::zero() {
            return None;
        }
        let sigma_m = variance.sqrt();
        if sigma_m > c.margin_m {
            return None;
        }

        Some(CommitSense {
            mach,
            q_inf,
            aided,
            sigma_m,
        })
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for ThrottleGuidance<R> {
    fn apply(
        &self,
        ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        // ── The commit: a rising edge, then a one-way latch on a carried scalar. ──
        let mut committed = self.committed(field);
        if !committed && let Some(sense) = self.corridor_holds(field) {
            committed = true;
            field.set_scalar(IGNITION_LATCH_FIELD, Vec::from([R::one()]));
            field.log_mut().add_entry(&alloc::format!(
                "ignition corridor committed at step {}: Mach {:?}, q {:?} Pa, nav {}, sigma {:?} m",
                ctx.step(),
                sense.mach,
                sense.q_inf,
                if sense.aided { "aided" } else { "dead-reckoning" },
                sense.sigma_m,
            ));
        }

        // ── The command: zero until committed, then the stopping-distance law. ──
        let throttle = if committed {
            let speed = field
                .scalar(self.speed_field)
                .and_then(|s| s.first().copied())
                .unwrap_or_else(R::zero);
            let altitude = field
                .scalar(self.altitude_field)
                .and_then(|s| s.first().copied())
                .unwrap_or_else(R::zero);
            let mass = field
                .scalar(self.mass_field)
                .and_then(|s| s.first().copied())
                .ok_or_else(|| {
                    PhysicsError::PhysicalInvariantBroken(
                        "ThrottleGuidance: a committed burn requires a carried \"mass\" scalar"
                            .into(),
                    )
                })?;

            // Ground contact and a non-positive gravity are the kernel's singularities; they
            // propagate rather than producing a throttle from an inadmissible state.
            let a_cmd = suicide_burn_deceleration_kernel(
                Speed::new(speed)?,
                Length::new(altitude)?,
                Acceleration::new(self.gravity)?,
            )?
            .value();

            if self.thrust <= R::zero() {
                return Err(PhysicsError::PhysicalInvariantBroken(
                    "ThrottleGuidance: full-throttle thrust must be positive".into(),
                ));
            }
            // The crate's linear convention, inverted: T = θ·T₁ ⇒ θ = m·a_cmd / T₁.
            let raw = mass * a_cmd / self.thrust;
            if raw < R::zero() {
                R::zero()
            } else if raw > R::one() {
                R::one()
            } else {
                raw
            }
        } else {
            R::zero()
        };

        field.set_throttle_action(throttle);
        Ok(())
    }
}
