/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::CfdScalar;
use deep_causality_core::EffectLog;
use deep_causality_haft::LogSize;

/// The owned result of a CfdFlow solver run: labeled observation series. The borrows
/// that produced it (manifold, solver) never escape `run`; only this owned `Report`
/// does (design D2). Shared by all three solver kinds (march, MMS-verify,
/// operator-study).
#[derive(Debug, Clone)]
pub struct Report<R: CfdScalar> {
    name: String,
    series: Vec<(String, Vec<R>)>,
    /// The final marched edge cochain (velocity 1-form), exposed so callers can compute bespoke
    /// diagnostics (centerline / streamfunction / edge-indexed probe) off the raw state.
    final_field: Option<Vec<R>>,
    /// The number of `EffectLog` entries the run accumulated (e.g. an uncertain-inflow march records
    /// dropout/intervention entries). `None` for runs with no effect log.
    log_entries: Option<usize>,
    /// The full provenance log a coupled run accumulated (regime transitions, nav-mode changes,
    /// bounded corrections, alternation markers) — the corridor [7] audit record. `None` for runs
    /// that carried no effect log.
    effect_log: Option<EffectLog>,
}

impl<R: CfdScalar> Report<R> {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            series: Vec::new(),
            final_field: None,
            log_entries: None,
            effect_log: None,
        }
    }

    pub(crate) fn add_series(&mut self, label: impl Into<String>, data: Vec<R>) {
        self.series.push((label.into(), data));
    }

    pub(crate) fn set_final_field(&mut self, field: Vec<R>) {
        self.final_field = Some(field);
    }

    pub(crate) fn set_log_entries(&mut self, count: usize) {
        self.log_entries = Some(count);
    }

    /// Attach the run's full provenance log (also sets the entry count).
    pub(crate) fn set_effect_log(&mut self, log: EffectLog) {
        self.log_entries = Some(log.len());
        self.effect_log = Some(log);
    }

    /// The number of `EffectLog` entries the run accumulated (uncertain-inflow dropout/intervention
    /// records), if the run carried an effect log.
    pub fn log_entries(&self) -> Option<usize> {
        self.log_entries
    }

    /// The full provenance log the run accumulated — regime transitions, nav-mode changes, bounded
    /// corrections, and counterfactual alternation markers — if the run carried one.
    pub fn effect_log(&self) -> Option<&EffectLog> {
        self.effect_log.as_ref()
    }

    /// The final marched edge cochain (velocity 1-form coefficients), if the run produced one.
    /// Bespoke diagnostics (Ghia centerline, streamfunction vortex centers, an edge-indexed wake
    /// probe) read this raw state and compute exactly as a hand-rolled march would.
    pub fn final_field(&self) -> Option<&[R]> {
        self.final_field.as_deref()
    }

    /// The case name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// A labeled observation series, if present.
    pub fn series(&self, label: &str) -> Option<&[R]> {
        self.series
            .iter()
            .find(|(l, _)| l == label)
            .map(|(_, d)| d.as_slice())
    }
}
