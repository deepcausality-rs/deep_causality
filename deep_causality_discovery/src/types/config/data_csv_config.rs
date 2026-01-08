/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

/// Configuration for loading data from a CSV file.
#[derive(Debug, Clone)]
pub struct CsvConfig {
    has_headers: bool,
    delimiter: u8,
    skip_rows: usize,
    columns: Option<Vec<String>>,
    file_path: Option<String>,
    target_index: Option<usize>,
    exclude_indices: Vec<usize>,
}

impl CsvConfig {
    /// Creates a new `CsvConfig`.
    pub fn new(
        has_headers: bool,
        delimiter: u8,
        skip_rows: usize,
        columns: Option<Vec<String>>,
        file_path: Option<String>,
        target_index: Option<usize>,
        exclude_indices: Vec<usize>,
    ) -> Self {
        Self {
            has_headers,
            delimiter,
            skip_rows,
            columns,
            file_path,
            target_index,
            exclude_indices,
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
            file_path: None,
            target_index: None,
            exclude_indices: vec![],
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

    /// The path to the CSV file, if known.
    pub fn file_path(&self) -> Option<&String> {
        self.file_path.as_ref()
    }

    /// The index of the target column.
    pub fn target_index(&self) -> Option<usize> {
        self.target_index
    }

    /// Indices of columns to exclude.
    pub fn exclude_indices(&self) -> &Vec<usize> {
        &self.exclude_indices
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
