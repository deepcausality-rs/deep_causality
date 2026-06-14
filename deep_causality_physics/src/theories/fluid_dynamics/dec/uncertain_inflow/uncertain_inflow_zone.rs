/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The immutable configuration of a sensor-fed inflow boundary patch.

use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_uncertain::ProbabilisticType;

use super::dropout_verbosity::DropoutVerbosity;
use super::uncertain_boundary_source::UncertainBoundarySource;

/// A sensor-fed inflow boundary patch (CFD Stage-4 — the first `MaybeUncertain` data zone).
///
/// The zone is the **immutable** half of the uncertain-inflow march (design D10: immutable data is
/// `Context`, the mutable last-good value is `State`). It names a prescribed-velocity wall — the
/// patch tagged as the inflow boundary — and delegates the per-step collapse of the
/// `MaybeUncertain<R>` reading to a cross-domain [`UncertainBoundarySource`]:
///
/// - **which boundary**: the wall perpendicular to `wall_axis` (`max_side` picks the far face),
///   driven with a tangential velocity on `flow_axis`, through the solver's existing moving-wall
///   lift — no new boundary machinery and no change to the stateless `step`;
/// - **the value policy**: the embedded [`UncertainBoundarySource`] owns the presence gate, the
///   collapse, the fallback, and the dropout-logging verbosity — exactly the mechanism reused
///   across domains.
///
/// `R` is the solver's precision; the zone is precision-generic because `MaybeUncertain<R>` and its
/// reduction are.
#[derive(Debug, Clone, Copy)]
pub struct UncertainInflowZone<R> {
    wall_axis: usize,
    max_side: bool,
    flow_axis: usize,
    source: UncertainBoundarySource<R>,
}

impl<R> UncertainInflowZone<R>
where
    R: RealField + FromPrimitive + ProbabilisticType + core::fmt::Debug,
{
    /// A zone driving the `wall_axis` wall (`max_side` face) with a tangential `flow_axis`
    /// velocity, falling back to `default_inflow` until the sensor first reads present. The value
    /// policy defaults follow [`UncertainBoundarySource::new`]; override them with the `with_*`
    /// builders. Axis validity is checked by the march driver against the concrete lattice.
    pub fn new(wall_axis: usize, max_side: bool, flow_axis: usize, default_inflow: R) -> Self {
        Self {
            wall_axis,
            max_side,
            flow_axis,
            source: UncertainBoundarySource::new(default_inflow),
        }
    }

    /// Sets the SPRT presence-gate parameters (delegated to the source).
    pub fn with_presence_gate(
        mut self,
        threshold: f64,
        confidence: f64,
        epsilon: f64,
        max_samples: usize,
    ) -> Self {
        self.source = self
            .source
            .with_presence_gate(threshold, confidence, epsilon, max_samples);
        self
    }

    /// Sets the sample count used to collapse a present sample to its mean (delegated).
    pub fn with_collapse_samples(mut self, collapse_samples: usize) -> Self {
        self.source = self.source.with_collapse_samples(collapse_samples);
        self
    }

    /// Sets the dropout-logging verbosity (delegated).
    pub fn with_verbosity(mut self, verbosity: DropoutVerbosity) -> Self {
        self.source = self.source.with_verbosity(verbosity);
        self
    }

    /// The wall-normal axis of the prescribed inflow wall.
    pub fn wall_axis(&self) -> usize {
        self.wall_axis
    }

    /// Whether the wall is the far (`max_side`) face of `wall_axis`.
    pub fn max_side(&self) -> bool {
        self.max_side
    }

    /// The tangential axis carrying the inflow velocity.
    pub fn flow_axis(&self) -> usize {
        self.flow_axis
    }

    /// The embedded cross-domain value source (presence gate, collapse, dropout policy).
    pub fn source(&self) -> &UncertainBoundarySource<R> {
        &self.source
    }

    /// The fallback inflow value (seeds the last-good state).
    pub fn default_inflow(&self) -> R {
        self.source.default_value()
    }
}
