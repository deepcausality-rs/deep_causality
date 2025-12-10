/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

/// Configuration for loading data from a Parquet file.
#[derive(Debug, Clone)]
pub struct ParquetConfig {
    columns: Option<Vec<String>>,
    batch_size: usize,
    file_path: Option<String>,
    target_index: Option<usize>,
    exclude_indices: Vec<usize>,
}

impl ParquetConfig {
    /// Creates a new `ParquetConfig`.
    pub fn new(
        columns: Option<Vec<String>>,
        batch_size: usize,
        file_path: Option<String>,
        target_index: Option<usize>,
        exclude_indices: Vec<usize>,
    ) -> Self {
        Self {
            columns,
            batch_size,
            file_path,
            target_index,
            exclude_indices,
        }
    }
}

impl ParquetConfig {
    /// An optional list of column names to select. If `None`, all columns are loaded.
    pub fn columns(&self) -> &Option<Vec<String>> {
        &self.columns
    }

    /// The number of rows to read at a time.
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    /// The path to the Parquet file, if known.
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

impl Default for ParquetConfig {
    fn default() -> Self {
        Self {
            columns: None,
            batch_size: 1024,
            file_path: None,
            target_index: None,
            exclude_indices: vec![],
        }
    }
}

impl fmt::Display for ParquetConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ParquetConfig(columns: {:?}, batch_size: {})",
            self.columns, self.batch_size
        )
    }
}
