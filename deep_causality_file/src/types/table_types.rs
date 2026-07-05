/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Typed numeric tables: the shape shared by the table reader and the result-table writer.
//!
//! A table is a list of named, unit-annotated columns and numeric rows. The on-disk form is
//! CSV with a two-row header: a column-name row, then an optional `#units` row, then data.
//! The writer emits exactly the shape the reader consumes, so a written table round-trips
//! with names, units, and `f64` bit patterns preserved.

/// One column's semantics: its name and its unit (empty string when the table carries none).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableColumn {
    name: String,
    unit: String,
}

impl TableColumn {
    /// A named column with a unit (pass `""` for a dimensionless or unit-less column).
    pub fn new(name: impl Into<String>, unit: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            unit: unit.into(),
        }
    }

    /// The column name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The column unit; empty when none was declared.
    pub fn unit(&self) -> &str {
        &self.unit
    }

    /// True when neither the name nor the unit contains the CSV delimiter (comma) or a newline,
    /// so this column serializes into the unescaped two-row-header shape and reads back
    /// unchanged. A column that fails this check would corrupt the round-trip; the table writer
    /// rejects such columns rather than emit a malformed file.
    pub fn is_delimiter_safe(&self) -> bool {
        !Self::has_delimiter(&self.name) && !Self::has_delimiter(&self.unit)
    }

    fn has_delimiter(s: &str) -> bool {
        s.contains(',') || s.contains('\n') || s.contains('\r')
    }
}

/// A typed numeric table: columns with semantics plus rectangular numeric rows in the working
/// scalar `R`. Construction validates rectangularity once, so a `NumericTable` in hand is
/// always well-shaped.
#[derive(Debug, Clone, PartialEq)]
pub struct NumericTable<R> {
    columns: Vec<TableColumn>,
    rows: Vec<Vec<R>>,
}

impl<R> NumericTable<R> {
    /// Build a table, validating that every row has exactly one value per column and that no
    /// two columns share a name. Returns `None` on a ragged shape or a duplicate column name,
    /// so a `NumericTable` in hand is always well-shaped and unambiguously addressable by name.
    pub fn new(columns: Vec<TableColumn>, rows: Vec<Vec<R>>) -> Option<Self> {
        let width = columns.len();
        if rows.iter().any(|r| r.len() != width) {
            return None;
        }
        // Name->index lookups are first-match, so duplicate names are silently ambiguous; reject.
        for (i, c) in columns.iter().enumerate() {
            if columns[i + 1..].iter().any(|other| other.name == c.name) {
                return None;
            }
        }
        Some(Self { columns, rows })
    }

    /// Build a table in one call from `(name, unit)` pairs and rows, with the same
    /// rectangularity validation as [`new`](Self::new).
    pub fn from_columns<const N: usize>(
        columns: [(&str, &str); N],
        rows: Vec<Vec<R>>,
    ) -> Option<Self> {
        Self::new(
            columns
                .into_iter()
                .map(|(n, u)| TableColumn::new(n, u))
                .collect(),
            rows,
        )
    }

    /// The column descriptors, in file order.
    pub fn columns(&self) -> &[TableColumn] {
        &self.columns
    }

    /// The data rows, each exactly `columns().len()` wide.
    pub fn rows(&self) -> &[Vec<R>] {
        &self.rows
    }

    /// The index of a named column, when present.
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.name == name)
    }

    /// The named column's values in row order, or an error naming the column when it is absent.
    /// Access by name, replacing positional row indexing.
    pub fn column(&self, name: &str) -> Result<Vec<R>, crate::DataLoadingError>
    where
        R: Clone,
    {
        let idx = self
            .column_index(name)
            .ok_or_else(|| crate::DataLoadingError::missing_column(name))?;
        Ok(self.rows.iter().map(|r| r[idx].clone()).collect())
    }

    /// Number of data rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// True when the table has no data rows.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}
