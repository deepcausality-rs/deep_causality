/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The immutable configuration of a sensor-fed inflow boundary patch.

use super::dropout_verbosity::DropoutVerbosity;

/// A sensor-fed inflow boundary patch (CFD Stage-4, Group C — the first `MaybeUncertain` data
/// zone).
///
/// The zone is the **immutable** half of the uncertain-inflow march (design D10: immutable data
/// is `Context`, the mutable last-good value is `State`). It names a prescribed-velocity wall —
/// the patch tagged as the inflow boundary — and the policy for collapsing each step's
/// `MaybeUncertain<R>` sensor reading to a single scalar inflow `R`:
///
/// - **which boundary**: the wall perpendicular to `wall_axis` (`max_side` picks the far face),
///   carrying a tangential velocity on `flow_axis` (`flow_axis != wall_axis`, else the value is a
///   forbidden wall-normal flux). The patch is driven through the solver's existing prescribed
///   moving-wall lift — no new boundary machinery and no change to the stateless `step`.
/// - **the presence gate**: `MaybeUncertain::lift_to_uncertain(threshold, confidence, epsilon,
///   max_samples)` decides, by SPRT, whether the sample is *present* enough to use;
/// - **the collapse**: a present sample's `Uncertain<R>` is reduced to a scalar by
///   `expected_value(collapse_samples)`;
/// - **the fallback**: `default_inflow` seeds the last-good value, used on a dropout before any
///   sample has ever been present;
/// - **the log policy**: [`DropoutVerbosity`] controls how densely dropouts are recorded.
///
/// The value type `R` is the solver's precision; the zone is precision-generic because
/// `MaybeUncertain<R>` and its reduction are (the `generalize-uncertain-over-realfield` line).
#[derive(Debug, Clone, Copy)]
pub struct UncertainInflowZone<R> {
    wall_axis: usize,
    max_side: bool,
    flow_axis: usize,
    threshold: f64,
    confidence: f64,
    epsilon: f64,
    max_samples: usize,
    collapse_samples: usize,
    default_inflow: R,
    verbosity: DropoutVerbosity,
}

impl<R: Copy> UncertainInflowZone<R> {
    /// A zone driving the `wall_axis` wall (`max_side` face) with a tangential `flow_axis`
    /// velocity, falling back to `default_inflow` until the sensor first reads present.
    ///
    /// The presence gate defaults to `threshold = 0.5`, `confidence = 0.95`, `epsilon = 0.05`,
    /// `max_samples = 1000`; the collapse to `1000` samples; the verbosity to
    /// [`DropoutVerbosity::EachDropout`]. Override them with the `with_*` builders. Axis validity
    /// (`< D`, `flow_axis != wall_axis`, non-periodic wall) is checked by the march driver against
    /// the concrete lattice.
    pub fn new(wall_axis: usize, max_side: bool, flow_axis: usize, default_inflow: R) -> Self {
        Self {
            wall_axis,
            max_side,
            flow_axis,
            threshold: 0.5,
            confidence: 0.95,
            epsilon: 0.05,
            max_samples: 1000,
            collapse_samples: 1000,
            default_inflow,
            verbosity: DropoutVerbosity::EachDropout,
        }
    }

    /// Sets the SPRT presence-gate parameters for `lift_to_uncertain`.
    pub fn with_presence_gate(
        mut self,
        threshold: f64,
        confidence: f64,
        epsilon: f64,
        max_samples: usize,
    ) -> Self {
        self.threshold = threshold;
        self.confidence = confidence;
        self.epsilon = epsilon;
        self.max_samples = max_samples;
        self
    }

    /// Sets the sample count used to collapse a present `Uncertain<R>` to its mean.
    pub fn with_collapse_samples(mut self, collapse_samples: usize) -> Self {
        self.collapse_samples = collapse_samples;
        self
    }

    /// Sets the dropout-logging verbosity.
    pub fn with_verbosity(mut self, verbosity: DropoutVerbosity) -> Self {
        self.verbosity = verbosity;
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

    /// The SPRT presence threshold probability.
    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    /// The SPRT confidence level.
    pub fn confidence(&self) -> f64 {
        self.confidence
    }

    /// The SPRT indifference region.
    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }

    /// The SPRT sampling budget for the presence gate.
    pub fn max_samples(&self) -> usize {
        self.max_samples
    }

    /// The sample count used to collapse a present sample to its mean.
    pub fn collapse_samples(&self) -> usize {
        self.collapse_samples
    }

    /// The fallback inflow value (seeds the last-good state).
    pub fn default_inflow(&self) -> R {
        self.default_inflow
    }

    /// The dropout-logging verbosity policy.
    pub fn verbosity(&self) -> DropoutVerbosity {
        self.verbosity
    }
}
