/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Result-table writer as a lazy [`IoAction`].
//!
//! Emits exactly the shape the table reader consumes: the column-name row, the `#units` row,
//! then data rows. Values are written with Rust's shortest round-trip `f64` formatting, so a
//! written table reads back with bit-identical values; the write happens at the display
//! boundary, where the house convention downcasts to `f64`.

use crate::DataLoadingError;
use crate::types::table_types::NumericTable;
use deep_causality_haft::IoAction;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// A lazy IO description that, when [`run`](IoAction::run), writes a [`NumericTable<f64>`] to
/// `path` in the two-row-header CSV shape. Construct with [`write_table`]; nothing touches the
/// filesystem before `.run()`.
pub struct WriteTable {
    path: PathBuf,
    table: NumericTable<f64>,
}

impl IoAction for WriteTable {
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
                // Shortest round-trip formatting: parsing this text recovers the exact bits.
                let _ = write!(out, "{v}");
            }
            out.push('\n');
        }

        fs::write(&self.path, out)?;
        Ok(())
    }
}

/// Describe (but do not perform) writing `table` to `path`.
pub fn write_table(path: impl AsRef<Path>, table: NumericTable<f64>) -> WriteTable {
    WriteTable {
        path: path.as_ref().to_path_buf(),
        table,
    }
}
