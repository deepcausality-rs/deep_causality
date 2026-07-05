/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Result-table writer as a lazy [`IoAction`], precision-generic over any [`TableScalar`].
//!
//! Emits exactly the shape the table reader consumes: the column-name row, the `#units` row,
//! then data rows. Each value is rendered through [`TableScalar::write_cell`], so a written
//! table reads back with identical bits at the written precision (`f64`, `f32`, or `Float106`);
//! the writer never downcasts a wider scalar to `f64`.

use crate::DataLoadingError;
use crate::traits::table_scalar::TableScalar;
use crate::types::table_types::NumericTable;
use deep_causality_haft::IoAction;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// A lazy IO description that, when [`run`](IoAction::run), writes a [`NumericTable<R>`] to
/// `path` in the two-row-header CSV shape, at the table's own precision `R`. Construct with
/// [`write_table`]; nothing touches the filesystem before `.run()`.
pub struct WriteTable<R: TableScalar> {
    path: PathBuf,
    table: NumericTable<R>,
}

impl<R: TableScalar> IoAction for WriteTable<R> {
    type Output = ();
    type Error = DataLoadingError;

    fn run(self) -> Result<(), DataLoadingError> {
        let mut out = String::new();
        let names: Vec<&str> = self.table.columns().iter().map(|c| c.name()).collect();
        out.push_str(&names.join(","));
        out.push('\n');

        // The #units row: the literal marker, then one unit cell per column.
        let mut units_row = String::from("#units");
        for c in self.table.columns() {
            let _ = write!(units_row, ",{}", c.unit());
        }
        out.push_str(&units_row);
        out.push('\n');

        for row in self.table.rows() {
            let mut first = true;
            for v in row {
                if !first {
                    out.push(',');
                }
                first = false;
                // Exact codec render: parsing this cell recovers the value at precision `R`.
                v.write_cell(&mut out);
            }
            out.push('\n');
        }

        fs::write(&self.path, out)?;
        Ok(())
    }
}

/// Describe (but do not perform) writing `table` to `path`, at the table's precision `R`.
pub fn write_table<R: TableScalar>(
    path: impl AsRef<Path>,
    table: NumericTable<R>,
) -> WriteTable<R> {
    WriteTable {
        path: path.as_ref().to_path_buf(),
        table,
    }
}
