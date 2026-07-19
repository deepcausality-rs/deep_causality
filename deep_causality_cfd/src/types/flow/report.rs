/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CfdScalar;
use deep_causality_core::EffectLog;
use deep_causality_haft::LogSize;

/// What a counterfactual branch's fork actually cost, recorded by the carrier at the moment the
/// branch was set up.
///
/// The state-fork's whole claim is that a branch is **O(1)**: it shares the paused marched tensor
/// and coupled field through `Arc` and takes a single copy-on-write clone at its first write, so a
/// roster of N branches costs one paused state rather than N copies. These fields are that claim
/// made checkable from a branch report, so a study can regress it instead of trusting it.
///
/// Recorded on every branch the carrier continues — [`CarrierPause::continue_with`], the fan-out
/// [`CarrierPause::continue_branches`] lowers onto it, and [`CarrierFork::continue_march`]. A report
/// from a plain (unforked) march carries `None`: nothing was forked, so there is nothing to claim.
///
/// [`CarrierPause::continue_with`]: crate::CarrierPause::continue_with
/// [`CarrierPause::continue_branches`]: crate::CarrierPause::continue_branches
/// [`CarrierFork::continue_march`]: crate::CarrierFork::continue_march
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForkEconomics {
    shares_fluid: bool,
    shares_field: bool,
    fluid_refs: usize,
    field_refs: usize,
}

impl ForkEconomics {
    pub(crate) fn new(
        shares_fluid: bool,
        shares_field: bool,
        fluid_refs: usize,
        field_refs: usize,
    ) -> Self {
        Self {
            shares_fluid,
            shares_field,
            fluid_refs,
            field_refs,
        }
    }

    /// Whether the branch started from the pause's marched tensor **by reference** — no tensor data
    /// copied at fork time.
    pub fn shares_fluid(&self) -> bool {
        self.shares_fluid
    }

    /// Whether the branch started from the pause's coupled field by reference.
    pub fn shares_field(&self) -> bool {
        self.shares_field
    }

    /// Live references to the shared marched tensor when this branch was set up. Greater than one
    /// is the positive evidence: the branch holds a *share* of the paused state rather than owning
    /// a copy of it.
    pub fn fluid_refs(&self) -> usize {
        self.fluid_refs
    }

    /// Live references to the shared coupled field when this branch was set up.
    pub fn field_refs(&self) -> usize {
        self.field_refs
    }

    /// The O(1)-fork claim in one call: both halves of the paused state entered this branch by
    /// reference, and each was genuinely shared rather than solely owned.
    pub fn is_o1(&self) -> bool {
        self.shares_fluid && self.shares_field && self.fluid_refs > 1 && self.field_refs > 1
    }
}

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
    /// What the fork cost, for a report produced by continuing a counterfactual branch. `None` for
    /// a plain march: nothing was forked.
    fork_economics: Option<ForkEconomics>,
}

impl<R: CfdScalar> Report<R> {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            series: Vec::new(),
            final_field: None,
            log_entries: None,
            effect_log: None,
            fork_economics: None,
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

    pub(crate) fn set_fork_economics(&mut self, economics: ForkEconomics) {
        self.fork_economics = Some(economics);
    }

    /// What this branch's fork cost, if this report came from continuing a forked branch. `None`
    /// for a plain march.
    pub fn fork_economics(&self) -> Option<ForkEconomics> {
        self.fork_economics
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
