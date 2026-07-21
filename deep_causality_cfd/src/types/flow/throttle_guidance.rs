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
    Acceleration, Length, PhysicsError, Speed, ignition_altitude_kernel,
    suicide_burn_deceleration_kernel,
};

/// The field scalar carrying the one-way ignition latch across steps and leg boundaries.
pub const IGNITION_LATCH_FIELD: &str = "ignition_committed";

/// The field scalar latching the start of a stopping burn, published by a guidance configured with
/// [`with_stopping_burn`](ThrottleGuidance::with_stopping_burn). Absent or zero while coasting.
pub const STOPPING_BURN_FIELD: &str = "stopping_burn";

/// The altitude (m) the stopping burn lit at, published when it latches.
///
/// Published for the same reason as the commit witnesses: a consumer comparing two beliefs' landing
/// decisions needs the altitude each one chose, and recovering it by splitting the prose entry ties
/// that comparison to the message's wording.
pub const STOPPING_BURN_ALTITUDE_FIELD: &str = "stopping_burn_altitude";
/// The flight speed (m/s) at which the stopping burn lit.
pub const STOPPING_BURN_SPEED_FIELD: &str = "stopping_burn_speed";

/// The step the ignition corridor committed on, published at the latching step.
///
/// The commit's sensed values are recorded twice: once as prose in the provenance log, and once as
/// these scalars. The prose is for a reader; the scalars are for a consumer. A gate that recovers a
/// commit witness by rendering the log and splitting the message depends on the message's wording and
/// on a scalar's `Debug` formatting, and reports a zero — an absent commit — when either changes.
pub const IGNITION_COMMIT_STEP_FIELD: &str = "ignition_commit_step";
/// The flight Mach the corridor sensed at the commit.
pub const IGNITION_COMMIT_MACH_FIELD: &str = "ignition_commit_mach";
/// The freestream dynamic pressure (Pa) the corridor sensed at the commit.
pub const IGNITION_COMMIT_Q_FIELD: &str = "ignition_commit_q";
/// Whether the navigation was aided at the commit (`1` aided, `0` dead-reckoning).
pub const IGNITION_COMMIT_AIDED_FIELD: &str = "ignition_commit_aided";
/// The navigated position uncertainty (m, one sigma) at the commit.
pub const IGNITION_COMMIT_SIGMA_FIELD: &str = "ignition_commit_sigma";

/// The world-published throttle command. When a committed world publishes it, it overrides the
/// guidance law — the counterfactual intervention seam, mirroring `"commanded_bank"`.
const COMMANDED_THROTTLE_FIELD: &str = "commanded_throttle";

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
    stopping_burn_margin: Option<R>,
    target_altitude: R,
    contact_speed: R,
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
            stopping_burn_margin: None,
            target_altitude: R::zero(),
            contact_speed: R::zero(),
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

    /// Fly the committed burn as a **stopping burn**: coast at zero throttle until the altitude
    /// falls to the ignition altitude, then burn. `margin` (m) is added to the stopping distance,
    /// sized by the caller the same way [`IgnitionCorridor::margin_m`] is.
    ///
    /// Without this, a guidance that commits high commands `a_cmd = v²/2h + g`, which for large `h`
    /// degenerates to `a_cmd ≈ g` — thrust balancing weight. The vehicle nulls its descent rate at
    /// altitude and then **hovers**, spending propellant to hold station instead of to land.
    ///
    /// Coast-then-burn is not a workaround for that but the *optimal* structure: Meditch (1964)
    /// showed the fuel-optimal control for the soft-landing problem is bang-bang — null thrust, then
    /// maximum thrust — so any sustained intermediate throttle is wasteful by construction. It is
    /// also what a real lander's minimum throttle forces: an engine whose floor thrust exceeds the
    /// landed weight *cannot* hover, and must arrive at zero velocity at zero altitude.
    ///
    /// Two admissions the caller should know about. The hold-off is evaluated against
    /// [`speed_field`](Self::new)'s **total** flight speed rather than the vertical component, so it
    /// ignites conservatively early while the flight path is still slanted. And when thrust-to-weight
    /// has fallen to one or below there is no coast to hold — no ignition altitude exists — so the
    /// stage burns rather than refusing, leaving the descent-rate axis to catch it.
    pub fn with_stopping_burn(mut self, margin: R) -> Self {
        self.stopping_burn_margin = Some(margin);
        self
    }

    /// Null the descent velocity at `target_altitude` (m) rather than at zero altitude.
    ///
    /// The stopping law targets the altitude where the vehicle should arrive stopped. Left at zero
    /// that is the geometric surface — but a lander arrives stopped at its **gear contact plane**,
    /// and a run that declares touchdown at a positive altitude floor samples the descent rate
    /// there. Aiming at zero while sampling at the floor reports the speed the vehicle still had one
    /// floor-height of braking short of the target, which is a formulation mismatch rather than a
    /// property of the vehicle: with a net deceleration of `a`, the ideal profile still carries
    /// `sqrt(2·a·h_floor)` at the floor.
    ///
    /// Below the target the law has arrived: the stage commands weight-balancing thrust and lets the
    /// vehicle settle at constant velocity rather than pushing the closed form through its
    /// singularity.
    pub fn with_target_altitude(mut self, target_altitude: R) -> Self {
        self.target_altitude = target_altitude;
        self
    }

    /// Arrive at the target plane still descending at `contact_speed` (m/s) instead of stopped.
    ///
    /// Landers do not null their velocity at the gear plane. Falcon 9 touches down near 2 m/s, and
    /// that is a requirement rather than a residual: its single landing engine at minimum throttle
    /// out-thrusts the nearly empty stage, so it **cannot** hover and must be flown to arrive
    /// moving. Deep-throttling vehicles that *can* hover still command a positive contact speed, to
    /// make firm contact and to stop station-keeping over a landing site they are not standing on.
    ///
    /// Realized through the same closed form rather than beside it: the burn must remove the kinetic
    /// energy down to the contact speed rather than to rest, so the kernel is asked to stop an
    /// effective speed `sqrt(v² − v_c²)`, which is zero exactly when the vehicle is already at or
    /// below the commanded contact condition.
    pub fn with_contact_speed(mut self, contact_speed: R) -> Self {
        self.contact_speed = contact_speed;
        self
    }

    /// The speed the stopping law must actually remove: everything above the contact speed.
    fn excess_speed(&self, speed: R) -> R {
        let excess = speed * speed - self.contact_speed * self.contact_speed;
        if excess <= R::zero() {
            R::zero()
        } else {
            excess.sqrt()
        }
    }

    /// True once the stopping burn has started and latched.
    fn burning(&self, field: &CoupledField<R>) -> bool {
        field
            .scalar(STOPPING_BURN_FIELD)
            .and_then(|s| s.first().copied())
            .is_some_and(|v| v > R::zero())
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
            // The typed witnesses, published once at the latching step and carried across leg
            // boundaries on the coupled field like every other scalar. A consumer reads the commit
            // without rendering the log.
            field.set_scalar(
                IGNITION_COMMIT_STEP_FIELD,
                Vec::from([R::from_f64(ctx.step() as f64).unwrap_or_else(R::zero)]),
            );
            field.set_scalar(IGNITION_COMMIT_MACH_FIELD, Vec::from([sense.mach]));
            field.set_scalar(IGNITION_COMMIT_Q_FIELD, Vec::from([sense.q_inf]));
            field.set_scalar(
                IGNITION_COMMIT_AIDED_FIELD,
                Vec::from([if sense.aided { R::one() } else { R::zero() }]),
            );
            field.set_scalar(IGNITION_COMMIT_SIGMA_FIELD, Vec::from([sense.sigma_m]));
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
        //
        // A world-published `"commanded_throttle"` **overrides** the law once committed. This is the
        // counterfactual command seam every other channel in the crate uses — the corridor's bank
        // guidance is nothing but a read of the published `"commanded_bank"` — and without it a
        // forked branch could not express an intervention: the guidance writes the throttle channel
        // every step, and the channel outranks the published scalar in the propulsion stages'
        // precedence, so the law would silently overwrite each branch's own command. The commit
        // still gates *whether* thrust is commanded, so branches differ in burn magnitude rather
        // than in whether they ignite.
        let throttle = if committed {
            if let Some(published) = field
                .scalar(COMMANDED_THROTTLE_FIELD)
                .and_then(|s| s.first().copied())
            {
                let bounded = if published < R::zero() {
                    R::zero()
                } else if published > R::one() {
                    R::one()
                } else {
                    published
                };
                field.set_throttle_action(bounded);
                return Ok(());
            }
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

            // ── The coast leg of the stopping burn: null thrust until the ignition altitude. ──
            //
            // Latched on its own scalar rather than recomputed as a live predicate: the ignition
            // altitude falls as propellant burns off (a_T = T/m rises), so a burn that started at
            // h_ign would find itself above the *new*, lower h_ign on the very next step and shut
            // down. The decision to start stopping is made once, like the corridor commit above.
            if let Some(margin) = self.stopping_burn_margin
                && !self.burning(field)
            {
                let a_t = self.thrust / mass;
                // Thrust-to-weight at or below one admits no coast: there is no altitude from which
                // this vehicle could still stop, so it burns and the descent-rate axis judges it.
                let hold_until = if a_t > self.gravity {
                    Some(
                        ignition_altitude_kernel(
                            Speed::new(self.excess_speed(speed))?,
                            Acceleration::new(a_t)?,
                            Acceleration::new(self.gravity)?,
                            Length::new(margin)?,
                        )?
                        .value(),
                    )
                } else {
                    None
                };

                // Measured above the target plane, for the same reason the law is.
                if let Some(h_ign) = hold_until
                    && altitude - self.target_altitude > h_ign
                {
                    field.set_throttle_action(R::zero());
                    return Ok(());
                }

                field.set_scalar(STOPPING_BURN_FIELD, Vec::from([R::one()]));
                field.set_scalar(STOPPING_BURN_ALTITUDE_FIELD, Vec::from([altitude]));
                field.set_scalar(STOPPING_BURN_SPEED_FIELD, Vec::from([speed]));
                field.log_mut().add_entry(&alloc::format!(
                    "stopping burn started at step {}: altitude {:?} m, speed {:?} m/s, thrust-to-weight {:?}",
                    ctx.step(),
                    altitude,
                    speed,
                    a_t / self.gravity,
                ));
            }

            // Ground contact and a non-positive gravity are the kernel's singularities; they
            // propagate rather than producing a throttle from an inadmissible state.
            // Height above the *target* plane: the law nulls the velocity where the vehicle is meant
            // to arrive stopped, not at the geometric surface below it.
            let height = altitude - self.target_altitude;
            let a_cmd = if height > R::zero() {
                suicide_burn_deceleration_kernel(
                    Speed::new(self.excess_speed(speed))?,
                    Length::new(height)?,
                    Acceleration::new(self.gravity)?,
                )?
                .value()
            } else if altitude > R::zero() {
                // At or below the commanded target plane but still above the surface: the burn has
                // arrived. Balance weight and settle rather than driving the closed form through a
                // singularity that is not physically present here.
                self.gravity
            } else {
                // Ground contact is a different situation entirely, and stays the kernel's
                // singularity: a guidance asked to command a burn from at or below the surface has
                // been asked for something inadmissible, and says so rather than inventing a
                // throttle. Re-entered through the kernel so the diagnosis stays the kernel's.
                suicide_burn_deceleration_kernel(
                    Speed::new(self.excess_speed(speed))?,
                    Length::new(altitude)?,
                    Acceleration::new(self.gravity)?,
                )?
                .value()
            };

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
