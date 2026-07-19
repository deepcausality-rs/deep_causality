/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The **flight-sensor producers** for the powered-descent envelope (change
//! `add-retropulsion-terminal-descent`, capability `flight-sensor-scalars`).
//!
//! [`CyberneticCorrect`](super::CyberneticCorrect) senses five scalars. Three of them
//! (`"heat_flux"`, `"g_load"`, `"propellant"`) come from a world's own stages. The remaining two —
//! `"q_inf"` and `"descent_rate"` — had **no producer anywhere outside the gate's own unit tests**,
//! which is why the descent-rate bound and the dynamic thrust-coefficient cap demonstrably work
//! under test while having no path to work in flight.
//!
//! The failure mode that motivates this stage is **fail-open**: the gate reads each sensed scalar
//! through a peak reduction folded from zero, so an absent producer reads as `0` — inside the
//! envelope for the descent-rate bound (it can never fire) and collapsing the dynamic ceiling to the
//! static one. Two safety axes report as enforcing while being unable to fire, and nothing in the
//! run says so.
//!
//! Both quantities are derived from what the compressible carrier already publishes:
//! - `q∞ = ½·ρ∞·V²` with `ρ∞ = n∞·m̄` from `"freestream_n"` (m⁻³) and `"flight_speed"` (m/s). The
//!   carrier holds the number density but carries **no** mean molecular mass, and the physics crate
//!   carries no air `m̄` either, so `m̄` is a **constructor parameter** rather than a smuggled
//!   species constant — the `FreestreamFeeds` example stage already performs the same `n·m̄`
//!   conversion.
//! - `ḣ = −(r·v)/|r|` from the stage-written `"truth_state"` (6 cells: position, then velocity).
//!   **Positive downward**, because the gate tests `descent_rate > max_descent_rate` after a maximum
//!   reduction — an ascent-positive convention would make the bound unreachable in exactly the
//!   regime it exists to protect.
//!
//! Both output field names are configurable, matching the gate's own `with_burn_sensing` sensing
//! configuration, so a world that renames one renames it on both sides.

use super::coupling::{CoupledField, PhysicsStage, StepContext};
use crate::CfdScalar;
use alloc::vec::Vec;
use deep_causality_physics::PhysicsError;

/// Publishes the two powered-descent sensor scalars the safety envelope reads but nothing else
/// produces: freestream dynamic pressure and descent rate.
///
/// Construct with [`new`](Self::new), supplying the mean molecular mass of the freestream gas
/// (kg per particle). Rename either output with [`with_field_names`](Self::with_field_names).
#[derive(Debug, Clone, Copy)]
pub struct FlightSensors<R: CfdScalar> {
    mean_molecular_mass: R,
    density_field: &'static str,
    speed_field: &'static str,
    truth_field: &'static str,
    q_field: &'static str,
    descent_rate_field: &'static str,
}

impl<R: CfdScalar> FlightSensors<R> {
    /// A sensor stage over the carrier's published freestream, with `mean_molecular_mass` the mass
    /// per freestream particle (kg) used to form `ρ∞ = n∞·m̄`.
    ///
    /// Defaults read `"freestream_n"`, `"flight_speed"` and `"truth_state"`, and publish `"q_inf"`
    /// and `"descent_rate"` — the names [`CyberneticCorrect`](super::CyberneticCorrect) senses by
    /// default.
    pub fn new(mean_molecular_mass: R) -> Self {
        Self {
            mean_molecular_mass,
            density_field: "freestream_n",
            speed_field: "flight_speed",
            truth_field: "truth_state",
            q_field: "q_inf",
            descent_rate_field: "descent_rate",
        }
    }

    /// Rename the two published outputs. Use when a world configures the gate's sensing away from
    /// the defaults, so producer and consumer stay in step.
    pub fn with_field_names(
        mut self,
        q_field: &'static str,
        descent_rate_field: &'static str,
    ) -> Self {
        self.q_field = q_field;
        self.descent_rate_field = descent_rate_field;
        self
    }

    /// Rename the three inputs read from the carrier and the truth propagator.
    pub fn with_input_names(
        mut self,
        density_field: &'static str,
        speed_field: &'static str,
        truth_field: &'static str,
    ) -> Self {
        self.density_field = density_field;
        self.speed_field = speed_field;
        self.truth_field = truth_field;
        self
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for FlightSensors<R> {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        // ── q∞ = ½·(n∞·m̄)·V², from the carrier's published freestream. ──
        let n_inf = field
            .scalar(self.density_field)
            .and_then(|s| s.first().copied());
        let speed = field
            .scalar(self.speed_field)
            .and_then(|s| s.first().copied());
        if let (Some(n_inf), Some(speed)) = (n_inf, speed) {
            let two = R::one() + R::one();
            let q_inf = (n_inf * self.mean_molecular_mass * speed * speed) / two;
            field.set_scalar(self.q_field, Vec::from([q_inf]));
        }

        // ── ḣ = −(r·v)/|r|, positive downward. ──
        // A partial truth state publishes nothing rather than a rate derived from half a vector.
        let truth = field.scalar(self.truth_field);
        if let Some(t) = truth
            && t.len() >= 6
        {
            let r = [t[0], t[1], t[2]];
            let v = [t[3], t[4], t[5]];
            let r_mag = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
            if r_mag > R::zero() {
                let r_dot_v = r[0] * v[0] + r[1] * v[1] + r[2] * v[2];
                let descent_rate = R::zero() - (r_dot_v / r_mag);
                field.set_scalar(self.descent_rate_field, Vec::from([descent_rate]));
            }
        }

        Ok(())
    }
}
