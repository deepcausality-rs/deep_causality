/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::CfdScalar;

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
}

impl<R: CfdScalar> Report<R> {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            series: Vec::new(),
            final_field: None,
        }
    }

    pub(crate) fn add_series(&mut self, label: impl Into<String>, data: Vec<R>) {
        self.series.push((label.into(), data));
    }

    pub(crate) fn set_final_field(&mut self, field: Vec<R>) {
        self.final_field = Some(field);
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
