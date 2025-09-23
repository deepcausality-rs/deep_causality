/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::DataError;
use crate::traits::process_data_loader::ProcessDataLoader;
use crate::types::config::DataLoaderConfig;
use deep_causality_tensor::CausalTensor;
use std::fs::File;

/// A concrete implementation of `ProcessDataLoader` for reading data from CSV files.
pub struct CsvDataLoader;

impl ProcessDataLoader for CsvDataLoader {
    fn load(&self, path: &str, config: &DataLoaderConfig) -> Result<CausalTensor<f64>, DataError> {
        if let DataLoaderConfig::Csv(csv_config) = config {
            let file = File::open(path).map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    DataError::FileNotFound(path.to_string())
                } else {
                    DataError::OsError(e.to_string())
                }
            })?;
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(csv_config.has_headers())
                .delimiter(csv_config.delimiter())
                .from_reader(file);

            let mut data = Vec::new();
            let mut width = 0;
            for result in rdr.records().skip(csv_config.skip_rows()) {
                let record = result?;
                if width == 0 {
                    width = record.len();
                }
                for field in record.iter() {
                    data.push(
                        field
                            .parse::<f64>()
                            .map_err(|e| DataError::OsError(e.to_string()))?,
                    );
                }
            }

            let height = if width == 0 { 0 } else { data.len() / width };
            CausalTensor::new(data, vec![height, width])
                .map_err(|e| DataError::OsError(e.to_string()))
        } else {
            Err(DataError::OsError(
                "Invalid config type for CsvDataLoader".to_string(),
            ))
        }
    }
}
