/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::CfdScalar;

/// The owned result of a Flow solver run: labeled observation series. The borrows
/// that produced it (manifold, solver) never escape `run`; only this owned `Report`
/// does (design D2). Shared by all three solver kinds (march, MMS-verify,
/// operator-study).
#[derive(Debug, Clone)]
pub struct Report<R: CfdScalar> {
    name: String,
    series: Vec<(String, Vec<R>)>,
}

impl<R: CfdScalar> Report<R> {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            series: Vec::new(),
        }
    }

    pub(crate) fn add_series(&mut self, label: impl Into<String>, data: Vec<R>) {
        self.series.push((label.into(), data));
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
