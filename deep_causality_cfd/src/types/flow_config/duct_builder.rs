/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `DuctConfigBuilder` — the fluent builder for a [`DuctConfig`]. Started by
//! [`CfdConfigBuilder::duct`](crate::CfdConfigBuilder), which takes the case name. Accumulates the
//! geometry, the inlet stagnation state, the back pressure, the resolution, and the stop condition,
//! and validates all of them at [`build`](DuctConfigBuilder::build); the built config is composed
//! and run by the [`CfdFlow`](crate::CfdFlow) DSL.

use crate::CfdScalar;
use crate::types::flow_config::duct_config::{DuctAreaProfile, DuctConfig};
use deep_causality_physics::PhysicsError;

/// Fluent builder for a [`DuctConfig`]. The area profile, the inlet stagnation state, the ratio of
/// specific heats, the back pressure, the cell count, and the stop condition are all required;
/// `build` reports the first missing section and then validates the values.
pub struct DuctConfigBuilder<R: CfdScalar> {
    name: String,
    profile: Option<DuctAreaProfile<R>>,
    inlet: Option<(R, R)>,
    gamma: Option<R>,
    back_pressure: Option<R>,
    cells: Option<usize>,
    stop: Option<(usize, R)>,
}

impl<R: CfdScalar> DuctConfigBuilder<R> {
    /// A fresh builder for the named case.
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            profile: None,
            inlet: None,
            gamma: None,
            back_pressure: None,
            cells: None,
            stop: None,
        }
    }

    /// The duct's cross-sectional area profile — a sampled `(x, A)` table or the analytic
    /// converging–diverging variant (required).
    pub fn profile(mut self, profile: DuctAreaProfile<R>) -> Self {
        self.profile = Some(profile);
        self
    }

    /// The inlet stagnation (reservoir) state: pressure `p₀` and temperature `T₀` in kelvin
    /// (required).
    pub fn inlet(mut self, p0: R, t0: R) -> Self {
        self.inlet = Some((p0, t0));
        self
    }

    /// The ratio of specific heats (required).
    pub fn gamma(mut self, gamma: R) -> Self {
        self.gamma = Some(gamma);
        self
    }

    /// The static back pressure at the exit plane, in the same unit as `p₀` (required).
    pub fn back_pressure(mut self, back_pressure: R) -> Self {
        self.back_pressure = Some(back_pressure);
        self
    }

    /// The finite-volume cell count (required).
    pub fn cells(mut self, cells: usize) -> Self {
        self.cells = Some(cells);
        self
    }

    /// The stop condition of the quasi-steady march: the step budget and the residual gate the
    /// march must settle below within it (required).
    pub fn stop(mut self, max_steps: usize, residual_tol: R) -> Self {
        self.stop = Some((max_steps, residual_tol));
        self
    }

    /// Finalize the configuration, validating the geometry and the physical state.
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] when a required section is missing; on a table
    /// with fewer than two points, non-ascending `x`, or a non-positive or non-finite area; on an
    /// analytic profile whose throat is not the strict minimum or whose length is not positive; on
    /// a `p0`, `t0`, or `back_pressure` that is not finite and positive; on
    /// `back_pressure >= p0` (nothing drives the flow); on `gamma` not finite and `> 1`; on
    /// `cells < 8` (the driver needs a resolvable throat); on a zero `max_steps`; or on a
    /// `residual_tol` that is not finite and positive.
    pub fn build(self) -> Result<DuctConfig<R>, PhysicsError> {
        let missing = |what: &str| {
            PhysicsError::PhysicalInvariantBroken(format!(
                "CfdConfigBuilder::duct: {what} is required"
            ))
        };
        let profile = self.profile.ok_or_else(|| missing("a profile"))?;
        let (p0, t0) = self.inlet.ok_or_else(|| missing("an inlet state"))?;
        let gamma = self.gamma.ok_or_else(|| missing("gamma"))?;
        let back_pressure = self
            .back_pressure
            .ok_or_else(|| missing("a back pressure"))?;
        let cells = self.cells.ok_or_else(|| missing("a cell count"))?;
        let (max_steps, residual_tol) = self.stop.ok_or_else(|| missing("a stop condition"))?;

        let positive = |x: R| x.is_finite() && x > R::zero();
        match &profile {
            DuctAreaProfile::Table(points) => {
                if points.len() < 2 {
                    return Err(PhysicsError::PhysicalInvariantBroken(
                        "DuctConfig area table needs at least two (x, area) points".into(),
                    ));
                }
                if points.iter().any(|&(x, a)| !x.is_finite() || !positive(a)) {
                    return Err(PhysicsError::PhysicalInvariantBroken(
                        "DuctConfig area table needs finite x and finite, positive areas".into(),
                    ));
                }
                if points.windows(2).any(|w| w[1].0 <= w[0].0) {
                    return Err(PhysicsError::PhysicalInvariantBroken(
                        "DuctConfig area table must be strictly ascending in x".into(),
                    ));
                }
            }
            DuctAreaProfile::ConvergingDiverging {
                inlet_area,
                throat_area,
                exit_area,
                length,
            } => {
                if !positive(*inlet_area) || !positive(*throat_area) || !positive(*exit_area) {
                    return Err(PhysicsError::PhysicalInvariantBroken(
                        "DuctConfig areas must be finite and positive".into(),
                    ));
                }
                if !(*throat_area < *inlet_area && *throat_area < *exit_area) {
                    return Err(PhysicsError::PhysicalInvariantBroken(
                        "DuctConfig throat_area must be the strict minimum (below inlet and exit)"
                            .into(),
                    ));
                }
                if !positive(*length) {
                    return Err(PhysicsError::PhysicalInvariantBroken(
                        "DuctConfig length must be finite and positive".into(),
                    ));
                }
            }
        }
        if !positive(p0) || !positive(t0) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DuctConfig stagnation state (p0, t0) must be finite and positive".into(),
            ));
        }
        if !(gamma.is_finite() && gamma > R::one()) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DuctConfig gamma must be finite and > 1".into(),
            ));
        }
        if !positive(back_pressure) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DuctConfig back_pressure must be finite and positive".into(),
            ));
        }
        if back_pressure >= p0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DuctConfig back_pressure must be below the stagnation pressure p0".into(),
            ));
        }
        if cells < 8 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DuctConfig needs at least 8 cells".into(),
            ));
        }
        if max_steps == 0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DuctConfig max_steps must be at least 1".into(),
            ));
        }
        if !positive(residual_tol) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DuctConfig residual_tol must be finite and positive".into(),
            ));
        }

        Ok(DuctConfig::new(
            self.name,
            profile,
            p0,
            t0,
            gamma,
            back_pressure,
            cells,
            max_steps,
            residual_tol,
        ))
    }
}
