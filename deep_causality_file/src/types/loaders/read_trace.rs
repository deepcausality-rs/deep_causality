/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Sensor-trace loader as a lazy [`IoAction`].
//!
//! The on-disk shape is the table shape with a timestamp first column: a header row
//! (`t,chan_a,chan_b,...`), an optional `#units` row, then sample rows. An **empty cell is a
//! missing sample** and loads as `None`; a non-empty cell that fails to parse is an error,
//! never a sentinel. Timestamps must parse in every row (a trace row without a time is
//! malformed). The uncertain lift (`MaybeUncertain` for presence, `Uncertain` for noise)
//! belongs to the consumer; this crate returns plain typed samples.

use crate::DataLoadingError;
use crate::types::trace_types::{SensorChannel, SensorTraceSet};
use deep_causality_algebra::RealField;
use deep_causality_haft::IoAction;
use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// The `#units` row marker (the exact first cell), shared with the table shape.
const UNITS_PREFIX: &str = "#units";

/// True when `line`'s first comma-separated cell is exactly the `#units` marker. An ordinary
/// comment whose first token merely starts with `#units` (e.g. `#units-note,...`) is not a
/// units row and stays a comment.
fn is_units_row(line: &str) -> bool {
    line.trim().split(',').next().map(str::trim) == Some(UNITS_PREFIX)
}

/// A lazy IO description that, when [`run`](IoAction::run), parses a per-channel sensor trace
/// into a [`SensorTraceSet`]. Construct with [`read_sensor_trace`]; nothing touches the
/// filesystem before `.run()`.
pub struct ReadSensorTrace<R> {
    path: PathBuf,
    _marker: PhantomData<R>,
}

impl<R> IoAction for ReadSensorTrace<R>
where
    R: RealField + From<f64>,
{
    type Output = SensorTraceSet<R>;
    type Error = DataLoadingError;

    fn run(self) -> Result<SensorTraceSet<R>, DataLoadingError> {
        parse_trace(&self.path)
    }
}

/// Describe (but do not perform) reading a sensor trace from `path`.
pub fn read_sensor_trace<R>(path: impl AsRef<Path>) -> ReadSensorTrace<R> {
    ReadSensorTrace {
        path: path.as_ref().to_path_buf(),
        _marker: PhantomData,
    }
}

fn parse_trace<R>(path: &Path) -> Result<SensorTraceSet<R>, DataLoadingError>
where
    R: RealField + From<f64>,
{
    let content = fs::read_to_string(path)?;
    let shown = path.display().to_string();

    let mut significant = content.lines().filter(|l| {
        let t = l.trim();
        !t.is_empty() && (!t.starts_with('#') || is_units_row(t))
    });

    let header = significant
        .next()
        .ok_or_else(|| DataLoadingError::table(&shown, 1, "missing header row"))?;
    let names: Vec<&str> = header.split(',').map(str::trim).collect();
    if names.len() < 2 {
        return Err(DataLoadingError::table(
            &shown,
            1,
            "a trace needs a timestamp column and at least one channel",
        ));
    }
    if names.iter().any(|n| n.is_empty()) {
        return Err(DataLoadingError::table(&shown, 1, "empty column name"));
    }
    // Reject duplicate column/channel names: channel lookup is first-match, so a repeated name
    // makes later channels unaddressable and can silently return the wrong series.
    for (i, n) in names.iter().enumerate() {
        if names[..i].contains(n) {
            return Err(DataLoadingError::table(
                &shown,
                1,
                format!("duplicate column name '{n}'"),
            ));
        }
    }

    let mut units: Vec<String> = vec![String::new(); names.len()];
    let mut timestamps: Vec<f64> = Vec::new();
    let mut samples: Vec<Vec<Option<R>>> = vec![Vec::new(); names.len() - 1];

    let mut row_no = 1usize;
    for line in significant {
        row_no += 1;
        let trimmed = line.trim();
        if is_units_row(trimmed) {
            if row_no != 2 {
                return Err(DataLoadingError::table(
                    &shown,
                    row_no,
                    "#units row is only valid directly after the header",
                ));
            }
            let cells: Vec<&str> = trimmed.split(',').map(str::trim).collect();
            // Cell 0 is the literal "#units" marker; cells 1.. carry one unit per column.
            if cells.len() != names.len() + 1 || cells[0] != UNITS_PREFIX {
                return Err(DataLoadingError::table(
                    &shown,
                    row_no,
                    format!(
                        "#units row needs the '#units' marker plus one unit per column \
                         ({} columns, got {} unit cells)",
                        names.len(),
                        cells.len().saturating_sub(1)
                    ),
                ));
            }
            units = cells[1..].iter().map(|u| (*u).to_string()).collect();
            continue;
        }

        let cells: Vec<&str> = trimmed.split(',').map(str::trim).collect();
        if cells.len() != names.len() {
            return Err(DataLoadingError::table(
                &shown,
                row_no,
                format!(
                    "ragged row: {} cells, header has {} columns",
                    cells.len(),
                    names.len()
                ),
            ));
        }
        let t = cells[0].parse::<f64>().map_err(|e| {
            DataLoadingError::table(
                &shown,
                row_no,
                format!("timestamp '{}' does not parse: {e}", cells[0]),
            )
        })?;
        timestamps.push(t);
        for (k, cell) in cells[1..].iter().enumerate() {
            if cell.is_empty() {
                samples[k].push(None);
            } else {
                let value = cell.parse::<f64>().map_err(|e| {
                    DataLoadingError::table(
                        &shown,
                        row_no,
                        format!(
                            "channel '{}': cannot parse '{cell}' as a number: {e}",
                            names[k + 1]
                        ),
                    )
                })?;
                samples[k].push(Some(R::from(value)));
            }
        }
    }

    let channels = names[1..]
        .iter()
        .zip(units[1..].iter())
        .zip(samples)
        .map(|((name, unit), s)| SensorChannel::new((*name).to_string(), unit.clone(), s))
        .collect();
    Ok(SensorTraceSet::new(timestamps, channels))
}
