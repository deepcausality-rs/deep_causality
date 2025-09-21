/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::config::csv_config::CsvConfig;
use crate::types::config::parquet_config::ParquetConfig;
use std::fmt;

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
