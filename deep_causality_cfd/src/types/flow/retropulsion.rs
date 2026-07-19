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

        // Retro-thrust acceleration along −v̂, composed onto (never over) the lift vector.
        let a_thrust = thrust / mass;
        field.add_aero_force([R::zero() - a_thrust, R::zero(), R::zero()]);
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
    /// Freestream static pressure, Pa.
    pub p_inf: R,
    /// Freestream Mach number (the Cordell envelope is [2, 4]).
    pub mach_inf: R,
    /// Freestream ratio of specific heats.
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
    q_inf: R,
    s_ref: R,
    nozzle: Option<PlumeNozzle<R>>,
}

impl<R: CfdScalar> PlumeObstruction<R> {
    /// A plume stage with full-throttle thrust `thrust` (N) and the A0 normalization pair `q_inf`
    /// (Pa) and `s_ref` (m²). No plume geometry is published until
    /// [`with_plume_geometry`](Self::with_plume_geometry) opts in.
    pub fn new(thrust: R, q_inf: R, s_ref: R) -> Self {
        Self {
            thrust,
            q_inf,
            s_ref,
            nozzle: None,
        }
    }

    /// Opt into publishing the analytic plume geometry each active step (state realism only — the
    /// force-channel decrement is unchanged either way).
    pub fn with_plume_geometry(mut self, nozzle: PlumeNozzle<R>) -> Self {
        self.nozzle = Some(nozzle);
        self
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for PlumeObstruction<R> {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(throttle) = active_throttle(field) else {
            return Ok(());
        };
        let thrust = throttle * self.thrust;

        // ── The A0 drag decrement: the committed drag authority. ──
        let c_t = srp_thrust_coefficient_kernel(
            Force::new(thrust)?,
            Pressure::new(self.q_inf)?,
            Area::new(self.s_ref)?,
        )?;
        let fraction = srp_preserved_drag_fraction_kernel(c_t)?;
        let axial_drag = field.aero_force().unwrap_or([R::zero(); 3])[0];
        field.add_aero_force([(fraction - R::one()) * axial_drag, R::zero(), R::zero()]);
        field.set_scalar("preserved_drag_fraction", Vec::from([fraction]));

        // ── Optional state-realism geometry for the carrier's re-imprint reader. ──
        if let Some(n) = self.nozzle {
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
                Pressure::new(n.p_inf)?,
                n.mach_inf,
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
