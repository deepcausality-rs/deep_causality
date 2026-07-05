/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Typed-row writer as a lazy [`IoAction`]: write a slice of [`TableRow`] where the column
//! schema and precision come from the row type, so column names live once on the row struct
//! rather than being repeated at the write site.
//!
//! The on-disk shape is the same two-row-header CSV the table reader consumes, so a `write_rows`
//! output reads back through [`read_rows`](crate::types::loaders::read_rows::read_rows).

use crate::DataLoadingError;
use crate::traits::table_row::TableRow;
use crate::traits::table_scalar::TableScalar;
use deep_causality_haft::IoAction;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// A lazy IO description that, when [`run`](IoAction::run), writes `rows` to `path` using the
/// schema and precision declared by `T: TableRow`. Construct with [`write_rows`].
pub struct WriteRows<T: TableRow> {
    path: PathBuf,
    rows: Vec<T>,
}

impl<T: TableRow> IoAction for WriteRows<T> {
    type Output = ();
    type Error = DataLoadingError;

    fn run(self) -> Result<(), DataLoadingError> {
        let schema = T::SCHEMA;
        let mut out = String::new();

        // Header: the schema's column names, then the #units row.
        let names: Vec<&str> = schema.iter().map(|(n, _)| *n).collect();
        out.push_str(&names.join(","));
        out.push('\n');
        let mut units_row = String::from("#units");
        for (_, unit) in schema {
            let _ = write!(units_row, ",{unit}");
        }
        out.push_str(&units_row);
        out.push('\n');

        for row in &self.rows {
            let cells = row.cells();
            if cells.len() != schema.len() {
                return Err(DataLoadingError::parse(
                    "write_rows",
                    format!(
                        "row produced {} cells but the schema has {} columns",
                        cells.len(),
                        schema.len()
                    ),
                ));
            }
            let mut first = true;
            for cell in &cells {
                if !first {
                    out.push(',');
                }
                first = false;
                cell.write_cell(&mut out);
            }
            out.push('\n');
        }

        fs::write(&self.path, out)?;
        Ok(())
    }
}

/// Describe (but do not perform) writing typed `rows` to `path`.
pub fn write_rows<T: TableRow>(path: impl AsRef<Path>, rows: Vec<T>) -> WriteRows<T> {
    WriteRows {
        path: path.as_ref().to_path_buf(),
        rows,
    }
}
