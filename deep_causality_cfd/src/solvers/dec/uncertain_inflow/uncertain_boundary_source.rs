/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The cross-domain uncertain boundary source: a sensor-fed, presence-gated, dropout-intervening
//! time-varying scalar value.

use alloc::format;

use deep_causality_core::EffectLog;
use deep_causality_haft::LogAddEntry;
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_uncertain::{MaybeUncertain, ProbabilisticType, UncertainError};

use super::dropout_verbosity::DropoutVerbosity;

/// Supplies the time-varying scalar value of a boundary (or, in principle, any parameter) from a
/// `MaybeUncertain<R>` stream — the cross-domain generalization of the Stage-4 uncertain-inflow
/// mechanism (CFD `add-boundary-zone-abstraction` D4).
///
/// Each step it presence-gates the sample (`lift_to_uncertain`) and, on success, collapses the
/// present `Uncertain<R>` to a scalar (`expected_value`), updating the caller's last-good value;
/// on a **dropout** (presence error, or a non-finite collapse) it returns the last-good value for
/// the caller to substitute (via `.intervene`) and records the dropout in the `EffectLog` at the
/// configured [`DropoutVerbosity`].
///
/// It depends only on `MaybeUncertain<R>` and the effect log — **not** on any fluid-dynamics type
/// — so the same source serves any sensor-fed time-varying parameter in any domain. Its bound is
/// the minimal `RealField + FromPrimitive + ProbabilisticType`, not the solver's `DecNsScalar`.
#[derive(Debug, Clone, Copy)]
pub struct UncertainBoundarySource<R> {
    threshold: f64,
    confidence: f64,
    epsilon: f64,
    max_samples: usize,
    collapse_samples: usize,
    default_value: R,
    verbosity: DropoutVerbosity,
}

impl<R> UncertainBoundarySource<R>
where
    R: RealField + FromPrimitive + ProbabilisticType + core::fmt::Debug,
{
    /// A source falling back to `default_value` until the sensor first reads present, with the
    /// default SPRT gate (`threshold 0.5`, `confidence 0.95`, `epsilon 0.05`, `max_samples 1000`),
    /// a `1000`-sample collapse, and [`DropoutVerbosity::EachDropout`].
    pub fn new(default_value: R) -> Self {
        Self {
            threshold: 0.5,
            confidence: 0.95,
            epsilon: 0.05,
            max_samples: 1000,
            collapse_samples: 1000,
            default_value,
            verbosity: DropoutVerbosity::EachDropout,
        }
    }

    /// Sets the SPRT presence-gate parameters.
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

    /// The fallback value (seeds the last-good).
    pub fn default_value(&self) -> R {
        self.default_value
    }

    /// The dropout-logging verbosity.
    pub fn verbosity(&self) -> DropoutVerbosity {
        self.verbosity
    }

    /// Resolves one sample against a mutable last-good value. On a present, finite collapse it
    /// updates `last_good` and returns `(value, false)`; on a dropout (presence error or a
    /// non-finite mean) it returns `(last_good, true)` without changing `last_good`.
    ///
    /// # Errors
    /// `UncertainError` for a sampling/gate failure that is **not** a presence error (those are
    /// dropouts, not errors).
    pub fn resolve(
        &self,
        sample: &MaybeUncertain<R>,
        last_good: &mut R,
    ) -> Result<(R, bool), UncertainError> {
        match sample.lift_to_uncertain(
            self.threshold,
            self.confidence,
            self.epsilon,
            self.max_samples,
        ) {
            Ok(present) => match present.expected_value(self.collapse_samples) {
                Ok(mean) if mean.is_finite() => {
                    *last_good = mean;
                    Ok((mean, false))
                }
                // A present-but-degenerate (non-finite) mean is treated as a dropout.
                Ok(_) => Ok((*last_good, true)),
                Err(e) => Err(e),
            },
            Err(UncertainError::PresenceError(_)) => Ok((*last_good, true)),
            Err(e) => Err(e),
        }
    }

    /// Records this step's dropout/recovery in `logs` per the verbosity. `in_dropout` is the
    /// previous step's dropout state; `value` is the resolved (fallback) value.
    pub fn record(
        &self,
        logs: &mut EffectLog,
        step: usize,
        dropout: bool,
        in_dropout: bool,
        value: R,
    ) {
        match self.verbosity {
            DropoutVerbosity::EachDropout => {
                if dropout {
                    logs.add_entry(&format!(
                        "uncertain boundary source: dropout at step {step} (fallback {value:?})"
                    ));
                }
            }
            DropoutVerbosity::Transitions => {
                if dropout && !in_dropout {
                    logs.add_entry(&format!(
                        "uncertain boundary source: dropout ONSET at step {step} (fallback {value:?})"
                    ));
                } else if !dropout && in_dropout {
                    logs.add_entry(&format!(
                        "uncertain boundary source: RECOVERY at step {step} (sensor present again)"
                    ));
                }
            }
        }
    }
}
