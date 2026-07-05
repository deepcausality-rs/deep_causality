/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The [`TableRow`] and [`FromTableRow`] traits: a typed result row declares its column schema
//! once, on the row struct itself, so a program's column names live next to the fields they
//! describe rather than being repeated at every write site.
//!
//! `write_rows` emits the schema's names and units from `TableRow`; `read_rows` matches a file's
//! header names to the schema and delivers each row's cells to [`FromTableRow::from_cells`] in
//! schema order, regardless of the file's column order.

use crate::traits::table_scalar::TableScalar;

/// A typed table row: its column schema and its cells, in the working precision `Scalar`.
///
/// `SCHEMA` gives the `(name, unit)` of each column in file order; `cells` projects the row to
/// exactly `SCHEMA.len()` values in the same order (checked once at write time).
pub trait TableRow {
    /// The working precision of this row's cells.
    type Scalar: TableScalar;

    /// The `(name, unit)` of each column, in file order. Pass `""` for a unit-less column.
    const SCHEMA: &'static [(&'static str, &'static str)];

    /// This row's cells, one per `SCHEMA` column, in schema order.
    fn cells(&self) -> Vec<Self::Scalar>;
}

/// The read-side inverse of [`TableRow`]: reconstruct a row from cells delivered in schema order.
///
/// `read_rows` matches the file header to [`TableRow::SCHEMA`] by name and reorders the file's
/// columns into schema order before calling `from_cells`, so the index of a value in `cells` is
/// always its `SCHEMA` position, never its position in the file.
pub trait FromTableRow: TableRow + Sized {
    /// Build a row from its cells, given in `SCHEMA` order. Returns `None` if the cells cannot
    /// form a valid row.
    fn from_cells(cells: &[Self::Scalar]) -> Option<Self>;
}
