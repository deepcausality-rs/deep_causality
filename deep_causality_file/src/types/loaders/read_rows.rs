/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Typed-row loader as a lazy [`IoAction`]: read a delimited table into `Vec<T>` where
//! `T: FromTableRow`. The file's header names are matched to the row type's
//! [`SCHEMA`](crate::traits::table_row::TableRow::SCHEMA), and each row's cells are delivered to
//! [`from_cells`](crate::traits::table_row::FromTableRow::from_cells) in schema order, so the
//! file may carry its columns in any order (and extra columns) without breaking the reader. A
//! required column absent from the file is an error naming that column.

use crate::DataLoadingError;
use crate::traits::table_row::{FromTableRow, TableRow};
use crate::types::loaders::read_table::read_table;
use deep_causality_haft::IoAction;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// A lazy IO description that, when [`run`](IoAction::run), parses `path` into `Vec<T>`.
/// Construct with [`read_rows`]; nothing touches the filesystem before `.run()`.
pub struct ReadRows<T> {
    path: PathBuf,
    _marker: PhantomData<T>,
}

impl<T> IoAction for ReadRows<T>
where
    T: FromTableRow,
{
    type Output = Vec<T>;
    type Error = DataLoadingError;

    fn run(self) -> Result<Vec<T>, DataLoadingError> {
        let table = read_table::<T::Scalar>(&self.path).run()?;

        // Map each schema column to its index in the file, by name. A required column the file
        // does not carry is an error naming that column.
        let schema = <T as TableRow>::SCHEMA;
        let mut order = Vec::with_capacity(schema.len());
        for (name, _) in schema {
            let idx = table
                .column_index(name)
                .ok_or_else(|| DataLoadingError::missing_column(*name))?;
            order.push(idx);
        }

        // Reorder each file row into schema order, then reconstruct the typed row.
        let mut out = Vec::with_capacity(table.len());
        for row in table.rows() {
            let cells: Vec<T::Scalar> = order.iter().map(|&i| row[i]).collect();
            let typed = T::from_cells(&cells).ok_or_else(|| {
                DataLoadingError::parse("read_rows", "row cells did not form a valid typed row")
            })?;
            out.push(typed);
        }
        Ok(out)
    }
}

/// Describe (but do not perform) reading typed rows `T` from `path`.
pub fn read_rows<T>(path: impl AsRef<Path>) -> ReadRows<T> {
    ReadRows {
        path: path.as_ref().to_path_buf(),
        _marker: PhantomData,
    }
}
