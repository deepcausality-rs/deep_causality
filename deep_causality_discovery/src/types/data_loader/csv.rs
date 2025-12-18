/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::DataLoadingError;
use crate::traits::data_loader::DataLoader;
use crate::types::config::DataLoaderConfig;
use deep_causality_tensor::CausalTensor;
use std::fs::File;

/// A concrete implementation of `ProcessDataLoader` for reading data from CSV files.
pub struct CsvDataLoader;

impl DataLoader for CsvDataLoader {
    fn load(
        &self,
        path: &str,
        config: &DataLoaderConfig,
    ) -> Result<CausalTensor<f64>, DataLoadingError> {
        if let DataLoaderConfig::Csv(csv_config) = config {
            let file = File::open(path).map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    DataLoadingError::FileNotFound(path.to_string())
                } else {
                    DataLoadingError::OsError(e.to_string())
                }
            })?;
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(csv_config.has_headers())
                .delimiter(csv_config.delimiter())
                .from_reader(file);

            let exclude_indices = csv_config.exclude_indices();
            let mut data = Vec::new();
            let mut width = 0;
            for result in rdr.records().skip(csv_config.skip_rows()) {
                let record = result?;
                let mut row_values = Vec::new();

                for (i, field) in record.iter().enumerate() {
                    if exclude_indices.contains(&i) {
                        continue;
                    }
                    row_values.push(
                        field
                            .parse::<f64>()
                            .map_err(|e| DataLoadingError::OsError(e.to_string()))?,
                    );
                }

                if width == 0 {
                    width = row_values.len();
                }
                data.extend(row_values);
            }

            let height = if width == 0 { 0 } else { data.len() / width };
            CausalTensor::new(data, vec![height, width])
                .map_err(|e| DataLoadingError::OsError(e.to_string()))
        } else {
            Err(DataLoadingError::OsError(
                "Invalid config type for CsvDataLoader".to_string(),
            ))
        }
    }
}
