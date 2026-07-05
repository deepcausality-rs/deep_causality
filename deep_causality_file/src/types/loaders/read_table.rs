/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Typed numeric-table loader as a lazy [`IoAction`].
//!
//! The on-disk shape: a column-name row, an optional `#units` row, then numeric rows,
//! comma-delimited. Lines that are empty or start with `#` (other than the `#units` row
//! directly after the header) are comments and are skipped. Each cell parses through the
//! [`TableScalar`] codec into the caller's scalar `R`, so a table written by the result-table
//! writer reads back with identical bits at the written precision, and a hand-authored
//! specification table of plain decimals loads at any precision. Malformed input is an error
//! naming the path and the offending significant row, never a default value.

use crate::DataLoadingError;
use crate::traits::table_scalar::TableScalar;
use crate::types::table_types::{NumericTable, TableColumn};
use deep_causality_haft::IoAction;
use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// The `#units` row prefix (first cell of the optional second header row).
const UNITS_PREFIX: &str = "#units";

/// A lazy IO description that, when [`run`](IoAction::run), parses a delimited numeric table
/// into a typed [`NumericTable`]. Construct with [`read_table`]; nothing touches the
/// filesystem before `.run()`.
pub struct ReadTable<R> {
    path: PathBuf,
    _marker: PhantomData<R>,
}

impl<R> IoAction for ReadTable<R>
where
    R: TableScalar,
{
    type Output = NumericTable<R>;
    type Error = DataLoadingError;

    fn run(self) -> Result<NumericTable<R>, DataLoadingError> {
        parse_table(&self.path)
    }
}

/// Describe (but do not perform) reading a typed numeric table from `path`.
pub fn read_table<R>(path: impl AsRef<Path>) -> ReadTable<R> {
    ReadTable {
        path: path.as_ref().to_path_buf(),
        _marker: PhantomData,
    }
}

fn parse_table<R>(path: &Path) -> Result<NumericTable<R>, DataLoadingError>
where
    R: TableScalar,
{
    let content = fs::read_to_string(path)?;
    let shown = path.display().to_string();

    // Significant lines: the header, the optional #units row, and data rows. Blank lines and
    // ordinary comments vanish before row counting, so error rows are stable under formatting.
    let mut significant = content.lines().enumerate().filter(|(_, l)| {
        let t = l.trim();
        !t.is_empty() && (!t.starts_with('#') || t.starts_with(UNITS_PREFIX))
    });

    let (_, header) = significant
        .next()
        .ok_or_else(|| DataLoadingError::table(&shown, 1, "missing header row"))?;
    let names: Vec<&str> = header.split(',').map(str::trim).collect();
    if names.iter().any(|n| n.is_empty()) {
        return Err(DataLoadingError::table(&shown, 1, "empty column name"));
    }

    let mut columns: Vec<TableColumn> = names.iter().map(|n| TableColumn::new(*n, "")).collect();

    let mut rows: Vec<Vec<R>> = Vec::new();
    let mut row_no = 1usize;
    for (_, line) in significant {
        row_no += 1;
        let trimmed = line.trim();
        if trimmed.starts_with(UNITS_PREFIX) {
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
            columns = names
                .iter()
                .zip(cells[1..].iter())
                .map(|(n, u)| TableColumn::new(*n, *u))
                .collect();
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
        let mut row = Vec::with_capacity(cells.len());
        for (cell, name) in cells.iter().zip(&names) {
            let value = R::parse_cell(cell).ok_or_else(|| {
                DataLoadingError::table(
                    &shown,
                    row_no,
                    format!("column '{name}': cannot parse '{cell}' as a number"),
                )
            })?;
            row.push(value);
        }
        rows.push(row);
    }

    NumericTable::new(columns, rows)
        .ok_or_else(|| DataLoadingError::table(&shown, row_no, "internal shape violation"))
}
