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
    Area, Force, PhysicsError, Pressure, propellant_mass_flow_kernel,
    srp_preserved_drag_fraction_kernel, srp_thrust_coefficient_kernel,
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
        field.throttle_action().or_else(|| {
            field
                .scalar("commanded_throttle")
                .and_then(|s| s.first().copied())
        })
    }
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
