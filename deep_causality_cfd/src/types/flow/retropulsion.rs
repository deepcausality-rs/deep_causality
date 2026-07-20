/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The inert-safe **A0 propulsion stub** behind the powered-descent coupling contract (change
//! `plasma-retropulsion-cfd-contracts`, capability `blackout-coupling-interface`).
//!
//! [`PropulsionStub`] follows the [`AeroBlackoutStub`](super::AeroBlackoutStub) precedent: a
//! library stage that satisfies the propulsion coupling seam the M3 production stages
//! (`RetroThrust`, `PlumeObstruction`) will fill, so downstream consumers build and validate
//! against a fixed seam. It reads the commanded throttle from the [`CoupledField`] throttle
//! channel (a guidance stage's write) or, absent one, the world-published `"commanded_throttle"`
//! scalar — the same seam as `"commanded_bank"`.
//!
//! **Inertness is the contract.** At commanded throttle ≤ 0 (or absent) the stub touches nothing:
//! no force write, no scalar mutation, no log entry — a coupled step with the stub composed is
//! bit-identical to one without it. The [`corridor-inheritance-guard`] harness rests on this.
//!
//! **Active path (throttle > 0)** exercises every seam on the pinned scalar names `"mass"`,
//! `"propellant"`, `"ignited"`:
//! - deplete `"propellant"` and reduce `"mass"` by `ṁ·Δt`, `ṁ` from the propellant-mass-flow
//!   kernel at thrust `T = throttle·thrust`;
//! - set `"ignited"`;
//! - add the retro-thrust acceleration `−T/m·v̂` (motion along `+x`, the corridor convention) onto
//!   the aero-force channel via [`add_aero_force`](CoupledField::add_aero_force), composing with
//!   the lift stage's vector rather than clobbering it;
//! - apply the **A0 drag decrement** — scale the axial drag already on the channel by the
//!   Jarvinen–Adams preserved-drag fraction at this `C_T` (`srp_thrust_coefficient` →
//!   `srp_preserved_drag_fraction`). Per the measured de-risk verdict
//!   (`openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md`, AMBER; the 2026-07-17 addendum
//!   measured both coupling models and pinned the missing collapse to the harness, not the model
//!   class), the A0 correlation is the **committed drag authority**: an M3 marched-layer imprint
//!   (the landed `ForcingRegion`), when composed for state realism, never replaces this closure.
//!
//! Pre-ignition, the world sets `"mass"` to the corridor's implied `CDA_OVER_M` constant-mass
//! bundle, so Act-1 force normalization is unchanged by carrying mass as state — a contract on the
//! world config the inheritance guard verifies, not something the stub enforces. The stub exists to
//! *validate the seams*, not to fly; M3 replaces it behind the same contract.

use super::coupling::{CoupledField, PhysicsStage, StepContext};
use crate::CfdScalar;
use alloc::vec::Vec;
use deep_causality_haft::LogAddEntry;
use deep_causality_physics::{
    Area, Force, Length, PhysicsError, Pressure, Temperature, cordell_braun_plume_boundary_kernel,
    propellant_mass_flow_kernel, srp_preserved_drag_fraction_kernel, srp_thrust_coefficient_kernel,
};

/// An inert-safe A0 propulsion stub satisfying the powered-descent coupling contract.
///
/// Parameters are the full-throttle thrust `T₁` (N), the specific impulse `Isp` (s), and the
/// freestream dynamic pressure `q∞` (Pa) and aerodynamic reference area `S_ref` (m²) the A0
/// thrust coefficient is normalized against. Effective thrust at commanded throttle `θ` is
/// `T = θ·T₁`.
#[derive(Debug, Clone, Copy)]
pub struct PropulsionStub<R: CfdScalar> {
    thrust: R,
    isp: R,
    q_inf: R,
    s_ref: R,
}

impl<R: CfdScalar> PropulsionStub<R> {
    /// A stub with full-throttle thrust `thrust` (N), specific impulse `isp` (s), and the A0
    /// normalization pair `q_inf` (Pa) and `s_ref` (m²).
    pub fn new(thrust: R, isp: R, q_inf: R, s_ref: R) -> Self {
        Self {
            thrust,
            isp,
            q_inf,
            s_ref,
        }
    }

    /// The commanded throttle for this step: the guidance-written throttle channel if set, else the
    /// world-published `"commanded_throttle"` scalar's first cell, else `None`.
    fn commanded_throttle(&self, field: &CoupledField<R>) -> Option<R> {
        commanded_throttle(field)
    }
}

/// The preserved-drag fraction the A0 correlation applied this step.
///
/// Absent when the closure stood down — outside its Mach band, or at zero throttle — so a consumer
/// distinguishes "no decrement applied" from "a decrement of this size applied" without inspecting a
/// log. A stale value left on the field would read as a live measurement.
pub const PRESERVED_DRAG_FRACTION_FIELD: &str = "preserved_drag_fraction";

/// The commanded throttle a propulsion stage acts on: the guidance-written throttle channel if set,
/// else the world-published `"commanded_throttle"` scalar's first cell, else `None`. Shared by every
/// stage behind the propulsion seam so the "channel overrides the published default" precedence is
/// identical across the stub and the production stages.
fn commanded_throttle<R: CfdScalar>(field: &CoupledField<R>) -> Option<R> {
    field.throttle_action().or_else(|| {
        field
            .scalar("commanded_throttle")
            .and_then(|s| s.first().copied())
    })
}

/// The active-path throttle for a propulsion stage: `Some(θ)` when a strictly positive throttle is
/// commanded, `None` when the stage must stay inert (the zero-throttle bit-identity contract).
fn active_throttle<R: CfdScalar>(field: &CoupledField<R>) -> Option<R> {
    match commanded_throttle(field) {
        Some(t) if t > R::zero() => Some(t),
        _ => None,
    }
}

/// The unit vector along the vehicle's flight velocity, from the carried `"truth_state"`
/// (6 cells: position, then velocity).
///
/// Retro thrust opposes the **velocity**, so its direction cannot be a fixed axis: the corridor's
/// motion is overwhelmingly tangential while its radial component is a descent, and the two swap
/// dominance over a descent. A stage that assumed one axis would, for most of a real trajectory,
/// point somewhere other than along the flight path — and on the radial axis it would push the
/// vehicle *toward* the planet rather than arresting it.
///
/// # Errors
/// [`PhysicsError::PhysicalInvariantBroken`] if the truth state is absent, short, or carries a
/// zero-magnitude velocity — thrust cannot be aimed without a direction to aim it against.
fn flight_velocity_unit<R: CfdScalar>(
    field: &CoupledField<R>,
    stage: &str,
) -> Result<[R; 3], PhysicsError> {
    let truth = field.scalar("truth_state").ok_or_else(|| {
        PhysicsError::PhysicalInvariantBroken(alloc::format!(
            "{stage}: active throttle requires a carried \"truth_state\" to resolve the flight direction"
        ))
    })?;
    if truth.len() < 6 {
        return Err(PhysicsError::PhysicalInvariantBroken(alloc::format!(
            "{stage}: \"truth_state\" must carry position and velocity (6 cells) to resolve the flight direction"
        )));
    }
    let v = [truth[3], truth[4], truth[5]];
    let mag = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if mag <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(alloc::format!(
            "{stage}: cannot resolve a flight direction from a zero-magnitude velocity"
        )));
    }
    Ok([v[0] / mag, v[1] / mag, v[2] / mag])
}

/// The carried vehicle mass under an active throttle.
///
/// # Errors
/// [`PhysicsError::PhysicalInvariantBroken`] if the `"mass"` scalar is absent or non-positive — a
/// mis-seeded burn world fails loudly rather than dividing by a missing mass.
fn carried_mass<R: CfdScalar>(field: &CoupledField<R>, stage: &str) -> Result<R, PhysicsError> {
    let mass = field
        .scalar("mass")
        .and_then(|s| s.first().copied())
        .ok_or_else(|| {
            PhysicsError::PhysicalInvariantBroken(alloc::format!(
                "{stage}: active throttle requires a carried \"mass\" scalar"
            ))
        })?;
    if mass <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(alloc::format!(
            "{stage}: carried mass must be positive under active throttle"
        )));
    }
    Ok(mass)
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for PropulsionStub<R> {
    fn apply(
        &self,
        ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        // Inert unless a positive throttle is commanded — touch nothing (the bit-identity contract).
        let throttle = match self.commanded_throttle(field) {
            Some(t) if t > R::zero() => t,
            _ => return Ok(()),
        };
        let thrust = throttle * self.thrust;

        // Carried mass is required to form the thrust acceleration; the contract rides it on the field.
        let mass = field
            .scalar("mass")
            .and_then(|s| s.first().copied())
            .ok_or_else(|| {
                PhysicsError::PhysicalInvariantBroken(
                    "PropulsionStub: active throttle requires a carried \"mass\" scalar".into(),
                )
            })?;
        if mass <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "PropulsionStub: carried mass must be positive under active throttle".into(),
            ));
        }

        // Propellant / mass depletion at ṁ from the specific-impulse kernel.
        let mdot = propellant_mass_flow_kernel(Force::new(thrust)?, self.isp)?.value();
        let dm = mdot * ctx.dt();
        if let Some(p) = field.scalar_mut("propellant")
            && let Some(p0) = p.first_mut()
        {
            *p0 -= dm;
        }
        if let Some(m) = field.scalar_mut("mass")
            && let Some(m0) = m.first_mut()
        {
            *m0 -= dm;
        }
        field.set_scalar("ignited", Vec::from([R::one()]));

        // A0 drag decrement: scale the axial drag on the channel by the preserved-drag fraction,
        // then add the retro-thrust acceleration `−T/m` along the motion axis (+x).
        let c_t = srp_thrust_coefficient_kernel(
            Force::new(thrust)?,
            Pressure::new(self.q_inf)?,
            Area::new(self.s_ref)?,
        )?;
        let fraction = srp_preserved_drag_fraction_kernel(c_t)?;
        let axial_drag = field.aero_force().unwrap_or([R::zero(); 3])[0];
        let decrement = (fraction - R::one()) * axial_drag;
        let a_thrust = thrust / mass;
        field.add_aero_force([decrement - a_thrust, R::zero(), R::zero()]);
        Ok(())
    }
}

/// The production **retro-thrust** stage (change `add-retropulsion-coupled-stages`, capability
/// `retro-thrust-stage`) — the thrust half of the M2 [`PropulsionStub`], productionized.
///
/// At a commanded throttle `θ > 0` it forms the retro-thrust acceleration `−(T/m)·v̂` with
/// `T = θ·thrust`, `m` the carried `"mass"` scalar, and `v̂` the flight-velocity direction (the
/// corridor's `+x` motion axis, so retro-thrust acts along `−x`), and composes it onto the
/// aero-force channel through [`add_aero_force`](CoupledField::add_aero_force) — never the
/// overwriting `set_aero_force`, so thrust adds to the lift stage's vector. It depletes
/// `"propellant"` and reduces `"mass"` by `ṁ·Δt` (`ṁ` from `propellant_mass_flow_kernel`) and sets
/// `"ignited"`. The thrust normalization uses the start-of-step mass, so it is consistent within the
/// step and grows as propellant burns off.
///
/// The thrust direction is the **carried flight velocity**, not a fixed axis: the acceleration is
/// `−(T/m)·v̂` with `v̂` read from the `"truth_state"` velocity each step. This is load-bearing
/// rather than cosmetic — a corridor-class trajectory is mostly tangential with a radial descent
/// component, so a hardcoded axis points along the flight path almost nowhere, and on the radial
/// axis it accelerates the descent instead of arresting it.
///
/// The stage is **strictly inert at throttle ≤ 0** (or absent): no force write, no scalar mutation,
/// no log entry — the [`PropulsionStub`] contract, so a burn-phase stack carries it from the start
/// and ignition stays a published-command event rather than a stack swap.
///
/// It composes **after** the ④-writing lift stage and **before** the force consumers (loads, the
/// truth propagator, the navigation kick), the M2 order contract. No navigation change is needed for
/// the IMU to feel the burn: the ESKF's specific-force term already reads the summed force channel.
///
/// The A0 drag decrement is **not** this stage's concern — [`PlumeObstruction`] owns it.
#[derive(Debug, Clone, Copy)]
pub struct RetroThrust<R: CfdScalar> {
    thrust: R,
    isp: R,
}

impl<R: CfdScalar> RetroThrust<R> {
    /// A retro-thrust stage with full-throttle thrust `thrust` (N) and specific impulse `isp` (s).
    pub fn new(thrust: R, isp: R) -> Self {
        Self { thrust, isp }
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for RetroThrust<R> {
    fn apply(
        &self,
        ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(throttle) = active_throttle(field) else {
            return Ok(());
        };
        let thrust = throttle * self.thrust;
        let mass = carried_mass(field, "RetroThrust")?;

        // Propellant / mass depletion at ṁ from the specific-impulse kernel.
        let mdot = propellant_mass_flow_kernel(Force::new(thrust)?, self.isp)?.value();
        let dm = mdot * ctx.dt();
        if let Some(p) = field.scalar_mut("propellant")
            && let Some(p0) = p.first_mut()
        {
            *p0 -= dm;
        }
        if let Some(m) = field.scalar_mut("mass")
            && let Some(m0) = m.first_mut()
        {
            *m0 -= dm;
        }
        field.set_scalar("ignited", Vec::from([R::one()]));

        // Retro-thrust acceleration along −v̂, composed onto (never over) the lift vector. The
        // direction comes from the carried flight velocity, not from a fixed axis: a retro burn
        // that does not oppose the velocity is not a retro burn.
        let v_hat = flight_velocity_unit(field, "RetroThrust")?;
        let a_thrust = thrust / mass;
        field.add_aero_force([
            R::zero() - a_thrust * v_hat[0],
            R::zero() - a_thrust * v_hat[1],
            R::zero() - a_thrust * v_hat[2],
        ]);
        Ok(())
    }
}

/// The nozzle + freestream description the analytic plume boundary needs
/// (`cordell_braun_plume_boundary_kernel`). A plain configuration record, like
/// [`AtmosphereRow`](crate::AtmosphereRow): the chamber pressure at full throttle scales linearly
/// with the commanded throttle, and the remaining fields are the fixed nozzle and freestream
/// constants. Supplying it to [`PlumeObstruction::with_plume_geometry`] opts a world into publishing
/// the plume geometry each step.
#[derive(Debug, Clone, Copy)]
pub struct PlumeNozzle<R: CfdScalar> {
    /// Chamber (stagnation) pressure at full throttle, Pa.
    pub chamber_pressure_max: R,
    /// Chamber (stagnation) temperature, K.
    pub chamber_temperature: R,
    /// Jet specific gas constant, J/(kg·K).
    pub r_specific: R,
    /// Jet ratio of specific heats (the Cordell envelope is [1.2, 1.4]).
    pub gamma_jet: R,
    /// Nozzle exit Mach number.
    pub exit_mach: R,
    /// Conical nozzle half-angle, rad.
    pub nozzle_half_angle_rad: R,
    /// Throat diameter, m.
    pub throat_diameter: R,
    /// Exit radius, m.
    pub exit_radius: R,
    /// Cone length, m.
    pub cone_length: R,
    /// Freestream ratio of specific heats.
    ///
    /// The freestream **static pressure** and **Mach number** are deliberately absent: they are
    /// sensed from the flown state each step, so the kernel's validity envelope tests the flight.
    /// Only the composition of the ambient gas is a fixed property of the world.
    pub gamma_inf: R,
}

/// The production **plume** stage (change `add-retropulsion-coupled-stages`, capability
/// `plume-obstruction-stage`) — the drag half of the M2 [`PropulsionStub`], productionized.
///
/// At a commanded throttle `θ > 0` it applies the **A0 drag decrement**: form `T = θ·thrust`, derive
/// `C_T = srp_thrust_coefficient_kernel(T, q∞, S_ref)`, read
/// `srp_preserved_drag_fraction_kernel(C_T)`, and scale the axial forebody drag already on the
/// aero-force channel by that preserved fraction. Per the measured de-risk verdict
/// (`openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md`, AMBER — the addendum measured both
/// coupling models and pinned the missing Jarvinen–Adams collapse to the harness, not the model
/// class), this correlation is the **committed drag authority** in flight; a marched-layer imprint is
/// never the force-channel closure. The applied fraction is published as
/// `"preserved_drag_fraction"` so a harness can cross-check it against M1's band.
///
/// **Optional state-realism geometry.** With [`with_plume_geometry`](Self::with_plume_geometry) the
/// stage also publishes the analytic plume's `"plume_max_radius"` and `"plume_penetration"` (from
/// `cordell_braun_plume_boundary_kernel` at that world's throttle-scaled chamber pressure). A
/// `PhysicsStage` cannot reach the marched layer, so the imprint itself is driven by the **carrier's
/// existing field-reading reconfiguration channel** — the same `pre_step` path that already reads the
/// stage-written `"truth_state"` to set the inflow strip and rebuild the marcher — which refreshes
/// the forcing region from these scalars behind its own opt-in. The published geometry never alters
/// the force-channel decrement.
///
/// The stage is **strictly inert at throttle ≤ 0** (or absent). Thrust is not this stage's concern —
/// [`RetroThrust`] owns it.
#[derive(Debug, Clone, Copy)]
pub struct PlumeObstruction<R: CfdScalar> {
    thrust: R,
    s_ref: R,
    q_field: &'static str,
    mach_field: &'static str,
    pressure_field: &'static str,
    mach_band: Option<(R, R)>,
    geometry_mach_band: Option<(R, R)>,
    nozzle: Option<PlumeNozzle<R>>,
}

impl<R: CfdScalar> PlumeObstruction<R> {
    /// A plume stage with full-throttle thrust `thrust` (N) and the A0 reference area `s_ref` (m²).
    ///
    /// The dynamic pressure is **sensed**, not supplied: it is read from `"q_inf"` each step, the
    /// same scalar [`CyberneticCorrect`](super::CyberneticCorrect)'s burn sensing reads, so the
    /// closure and the envelope's dynamic `C_T` cap describe one physical state.
    ///
    /// No plume geometry is published until [`with_plume_geometry`](Self::with_plume_geometry) opts
    /// in, and the correlation applies at every Mach until
    /// [`with_mach_band`](Self::with_mach_band) bounds it.
    pub fn new(thrust: R, s_ref: R) -> Self {
        Self {
            thrust,
            s_ref,
            q_field: "q_inf",
            mach_field: "flight_mach",
            pressure_field: "p_inf",
            mach_band: None,
            geometry_mach_band: None,
            nozzle: None,
        }
    }

    /// Rename the sensed scalars, matching a world that configured the gate's sensing away from the
    /// defaults so producer and both consumers stay in step.
    pub fn with_sensing(
        mut self,
        q_field: &'static str,
        mach_field: &'static str,
        pressure_field: &'static str,
    ) -> Self {
        self.q_field = q_field;
        self.mach_field = mach_field;
        self.pressure_field = pressure_field;
        self
    }

    /// Bound the correlation to the flight Mach band it was measured over.
    ///
    /// The Jarvinen–Adams dataset spans Mach 0.4–2.0 and its mechanism is **bow-shock displacement**:
    /// the plume pushes the standoff shock away and the high post-shock pressure that was
    /// decelerating the forebody is replaced by low-pressure recirculating plume gas. Below the
    /// dataset's Mach floor there is no bow shock to displace, so the correlation describes nothing
    /// there — carrying it down deletes most of a subsonic vehicle's aerodynamic drag on the strength
    /// of a supersonic interaction. Outside the band the stage stands down and records that it did.
    pub fn with_mach_band(mut self, mach_min: R, mach_max: R) -> Self {
        self.mach_band = Some((mach_min, mach_max));
        self
    }

    /// Opt into publishing the analytic plume geometry each active step (state realism only — the
    /// force-channel decrement is unchanged either way).
    pub fn with_plume_geometry(mut self, nozzle: PlumeNozzle<R>) -> Self {
        self.nozzle = Some(nozzle);
        self
    }

    /// Bound the **geometry** publication to the plume model's own validated Mach range, separately
    /// from the drag correlation's band.
    ///
    /// The two bands are separate because the two models are: the drag correlation is measured over
    /// Mach 0.4–2.0 and the plume-boundary model is validated over Mach 2–4, so a descent that flies
    /// the correlation's band is outside the geometry model's for almost all of it. Declaring both at
    /// the call site makes that disjointness legible instead of leaving it to be discovered when the
    /// kernel refuses mid-flight — or, worse, hidden by handing the kernel a constant that sits
    /// inside its envelope while the vehicle does not.
    ///
    /// Outside the band the stage publishes no geometry and records the crossing once. Inside it, a
    /// kernel refusal still propagates: the band says where the model is asked to apply, and the
    /// kernel remains the authority on whether it can.
    pub fn with_geometry_mach_band(mut self, mach_min: R, mach_max: R) -> Self {
        self.geometry_mach_band = Some((mach_min, mach_max));
        self
    }

    /// Whether the sensed flight Mach lies inside the geometry model's declared band.
    fn geometry_applies(&self, field: &CoupledField<R>) -> bool {
        let Some((lo, hi)) = self.geometry_mach_band else {
            return true;
        };
        field
            .scalar(self.mach_field)
            .and_then(|s| s.first().copied())
            .is_some_and(|m| m >= lo && m <= hi)
    }

    /// Whether the sensed flight Mach lies inside the configured applicability band. An unbounded
    /// stage applies everywhere; an absent Mach scalar under a configured band is outside it, because
    /// an absent sensor is a condition unmet rather than a condition waived.
    fn within_band(&self, field: &CoupledField<R>) -> bool {
        let Some((lo, hi)) = self.mach_band else {
            return true;
        };
        field
            .scalar(self.mach_field)
            .and_then(|s| s.first().copied())
            .is_some_and(|m| m >= lo && m <= hi)
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for PlumeObstruction<R> {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(throttle) = active_throttle(field) else {
            // Inert at zero throttle — but a *stale* fraction is not inertness. A world that was
            // burning and then shut down would otherwise carry its last decrement forward, and a
            // consumer reading it would report a plume that is no longer there. Taking the scalar is
            // a no-op for a world that never published one, so the bit-identity contract holds.
            if field.take_scalar(PRESERVED_DRAG_FRACTION_FIELD).is_some() {
                field.log_mut().add_entry(
                    "SRP drag closure inert: no throttle commanded, no decrement applied",
                );
            }
            return Ok(());
        };

        // ── Applicability first: outside the band the correlation describes nothing. ──
        //
        // Logged on the crossing rather than every step, so a leg spent outside the band leaves one
        // entry rather than one per step. The marker is the published fraction's presence: a step
        // that stood down clears it, so a consumer cannot read a stale fraction as a live one.
        if !self.within_band(field) {
            if field.take_scalar(PRESERVED_DRAG_FRACTION_FIELD).is_some() {
                field.log_mut().add_entry(
                    "SRP drag closure stood down: flight Mach outside the Jarvinen-Adams band, \
                     no decrement applied",
                );
            }
            return Ok(());
        }

        let thrust = throttle * self.thrust;

        // ── The A0 drag decrement: the committed drag authority. ──
        //
        // The dynamic pressure is the sensed one. Taking it at construction let this closure and the
        // safety gate's dynamic `C_T` cap normalize the same coefficient against two different
        // pressures in the same step, so the closure could evaluate the correlation deep in its
        // shallow range while the gate simultaneously judged the vehicle to be at its stability
        // limit. An absent or non-positive sensor is an error rather than a fallback: a fallback is
        // how a second normalization survives unnoticed.
        let q_inf = field
            .scalar(self.q_field)
            .and_then(|s| s.first().copied())
            .ok_or_else(|| {
                PhysicsError::PhysicalInvariantBroken(alloc::format!(
                    "PlumeObstruction: active throttle requires a sensed \"{}\" to normalize C_T",
                    self.q_field
                ))
            })?;
        let c_t = srp_thrust_coefficient_kernel(
            Force::new(thrust)?,
            Pressure::new(q_inf)?,
            Area::new(self.s_ref)?,
        )?;
        let fraction = srp_preserved_drag_fraction_kernel(c_t)?;

        // The plume destroys *aerodynamic drag*, which acts along −v̂ — so the decrement scales the
        // along-velocity component of the force channel and leaves the lateral (lift) components
        // alone. Reading a fixed axis instead would scale whatever happened to sit on x, which for a
        // corridor-class trajectory is mostly the radial term rather than the drag.
        //
        // This stage must therefore compose **before** the thrust stage: both write along −v̂, and a
        // decrement applied after thrust would scale the thrust term too, which the correlation says
        // nothing about.
        let v_hat = flight_velocity_unit(field, "PlumeObstruction")?;
        let force = field.aero_force().unwrap_or([R::zero(); 3]);
        let axial_drag = force[0] * v_hat[0] + force[1] * v_hat[1] + force[2] * v_hat[2];
        let scale = (fraction - R::one()) * axial_drag;
        field.add_aero_force([scale * v_hat[0], scale * v_hat[1], scale * v_hat[2]]);
        field.set_scalar(PRESERVED_DRAG_FRACTION_FIELD, Vec::from([fraction]));

        // ── Optional state-realism geometry for the carrier's re-imprint reader. ──
        if !self.geometry_applies(field) {
            if field.take_scalar("plume_max_radius").is_some() {
                let _ = field.take_scalar("plume_penetration");
                field.log_mut().add_entry(
                    "plume geometry stood down: flight Mach outside the Cordell-Braun validated \
                     envelope, no boundary published",
                );
            }
            return Ok(());
        }
        if let Some(n) = self.nozzle {
            // The freestream the jet expands against is the **sensed** one. Frozen constants make
            // the kernel's own validity envelope test the constant rather than the flight, so a leg
            // that leaves the envelope still receives geometry.
            let p_inf = field
                .scalar(self.pressure_field)
                .and_then(|s| s.first().copied())
                .ok_or_else(|| {
                    PhysicsError::PhysicalInvariantBroken(alloc::format!(
                        "PlumeObstruction: plume geometry requires a sensed \"{}\"",
                        self.pressure_field
                    ))
                })?;
            let mach_inf = field
                .scalar(self.mach_field)
                .and_then(|s| s.first().copied())
                .ok_or_else(|| {
                    PhysicsError::PhysicalInvariantBroken(alloc::format!(
                        "PlumeObstruction: plume geometry requires a sensed \"{}\"",
                        self.mach_field
                    ))
                })?;
            let geometry = cordell_braun_plume_boundary_kernel(
                Pressure::new(throttle * n.chamber_pressure_max)?,
                Temperature::new(n.chamber_temperature)?,
                n.r_specific,
                n.gamma_jet,
                n.exit_mach,
                n.nozzle_half_angle_rad,
                Length::new(n.throat_diameter)?,
                Length::new(n.exit_radius)?,
                Length::new(n.cone_length)?,
                Pressure::new(p_inf)?,
                mach_inf,
                n.gamma_inf,
            )?;
            field.set_scalar(
                "plume_max_radius",
                Vec::from([geometry.max_radius().value()]),
            );
            field.set_scalar(
                "plume_penetration",
                Vec::from([geometry.penetration_length().value()]),
            );
        }
        Ok(())
    }
}
