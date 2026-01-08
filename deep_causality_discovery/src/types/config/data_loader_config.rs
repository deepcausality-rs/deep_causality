/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CsvConfig, ParquetConfig};
use std::fmt;

/// An enum representing the configuration for a specific data loader.
///
/// This allows the CDL pipeline to be configured for different data sources
/// like CSV or Parquet files.
#[derive(Debug, Clone)]
pub enum DataLoaderConfig {
    Csv(CsvConfig),
    Parquet(ParquetConfig),
}

impl fmt::Display for DataLoaderConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataLoaderConfig::Csv(c) => write!(f, "DataLoaderConfig::Csv({})", c),
            DataLoaderConfig::Parquet(c) => write!(f, "DataLoaderConfig::Parquet({})", c),
        }
    }
}
