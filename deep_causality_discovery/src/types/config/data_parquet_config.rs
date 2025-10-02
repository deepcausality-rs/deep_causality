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
}

impl ParquetConfig {
    /// Creates a new `ParquetConfig`.
    pub fn new(columns: Option<Vec<String>>, batch_size: usize) -> Self {
        Self {
            columns,
            batch_size,
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
}

impl Default for ParquetConfig {
    fn default() -> Self {
        Self {
            columns: None,
            batch_size: 1024,
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
