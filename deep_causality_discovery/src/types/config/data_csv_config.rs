/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

/// Configuration for loading data from a CSV file.
#[derive(Debug, Clone)]
pub struct CsvConfig {
    has_headers: bool,
    delimiter: u8,
    skip_rows: usize,
    columns: Option<Vec<String>>,
}

impl CsvConfig {
    /// Creates a new `CsvConfig`.
    pub fn new(
        has_headers: bool,
        delimiter: u8,
        skip_rows: usize,
        columns: Option<Vec<String>>,
    ) -> Self {
        Self {
            has_headers,
            delimiter,
            skip_rows,
            columns,
        }
    }
}

impl Default for CsvConfig {
    fn default() -> Self {
        Self {
            has_headers: true,
            delimiter: b',',
            skip_rows: 0,
            columns: None,
        }
    }
}

impl CsvConfig {
    /// Whether the CSV file has a header row.
    pub fn has_headers(&self) -> bool {
        self.has_headers
    }

    /// The delimiter used to separate fields in the CSV file.
    pub fn delimiter(&self) -> u8 {
        self.delimiter
    }

    /// The number of rows to skip at the beginning of the file.
    pub fn skip_rows(&self) -> usize {
        self.skip_rows
    }

    /// An optional list of column names to select. If `None`, all columns are loaded.
    pub fn columns(&self) -> &Option<Vec<String>> {
        &self.columns
    }
}

impl fmt::Display for CsvConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CsvConfig(headers: {}, delimiter: {}, skip: {}, columns: {:?})",
            self.has_headers, self.delimiter as char, self.skip_rows, self.columns
        )
    }
}
