/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CSV application of the IO effect: deferred [`WriteCsv`] actions for CFD outputs.
//!
//! These build on the `std`-only core file action
//! [`write_csv`](deep_causality_core::write_csv); each is a lazy description that performs no IO
//! until `run`. They render with each scalar's [`Display`](core::fmt::Display), so the output is
//! plain, unquoted CSV.

use crate::types::CfdScalar;
use crate::types::flow::Report;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use deep_causality_core::{WriteCsv, write_csv};
use std::path::PathBuf;

impl<R: CfdScalar> Report<R> {
    /// Describe writing the named observation `series` as CSV columns under `labels` (runs only at
    /// the edge). A missing label contributes an empty column; the row count is the shortest present
    /// column, so ragged series never panic.
    pub fn write_series_csv(&self, path: impl Into<PathBuf>, labels: &[&str]) -> WriteCsv {
        let header: Vec<String> = labels.iter().map(|l| l.to_string()).collect();
        let cols: Vec<&[R]> = labels
            .iter()
            .map(|l| self.series(l).unwrap_or(&[]))
            .collect();
        let n_rows = cols.iter().map(|c| c.len()).min().unwrap_or(0);
        let rows: Vec<Vec<String>> = (0..n_rows)
            .map(|i| cols.iter().map(|c| c[i].to_string()).collect())
            .collect();
        write_csv(path, header, rows)
    }
}

/// Describe writing an `(x, y)` series as a two-column CSV under `header` (runs only at the edge).
/// Convenient for a probe time-series such as `(t, v_probe)`. Generic over the scalar `R` so callers
/// render at their working precision without downcasting; each value is formatted through its
/// `Display`.
pub fn write_xy_csv<R: CfdScalar>(
    path: impl Into<PathBuf>,
    header: [&str; 2],
    series: &[(R, R)],
) -> WriteCsv {
    let header = vec![header[0].to_string(), header[1].to_string()];
    let rows = series
        .iter()
        .map(|(x, y)| vec![x.to_string(), y.to_string()])
        .collect();
    write_csv(path, header, rows)
}
