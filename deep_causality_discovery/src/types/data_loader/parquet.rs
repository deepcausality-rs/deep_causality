/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::DataLoadingError;
use crate::traits::data_loader::DataLoader;
use crate::types::config::DataLoaderConfig;
use deep_causality_tensor::CausalTensor;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;
use std::fs::File;
use std::path::Path;

/// A concrete implementation of `ProcessDataLoader` for reading data from Parquet files.
pub struct ParquetDataLoader;

impl DataLoader for ParquetDataLoader {
    fn load(
        &self,
        path: &str,
        config: &DataLoaderConfig,
    ) -> Result<CausalTensor<f64>, DataLoadingError> {
        if let DataLoaderConfig::Parquet(parquet_config) = config {
            let file = File::open(Path::new(path))
                .map_err(|e| DataLoadingError::OsError(e.to_string()))?;
            let reader = SerializedFileReader::new(file)?;

            let mut data = Vec::new();
            let mut width = 0;

            let iter = reader.get_row_iter(None)?;

            let exclude_indices = parquet_config.exclude_indices();

            for record_result in iter {
                let record = record_result?;
                let mut row_values = Vec::new();

                for (i, (name, field)) in record.get_column_iter().enumerate() {
                    if exclude_indices.contains(&i) {
                        continue;
                    }

                    let val = match field {
                        Field::Bool(v) => {
                            if *v {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        Field::Byte(v) => *v as f64,
                        Field::Short(v) => *v as f64,
                        Field::Int(v) => *v as f64,
                        Field::Long(v) => *v as f64,
                        Field::UByte(v) => *v as f64,
                        Field::UShort(v) => *v as f64,
                        Field::UInt(v) => *v as f64,
                        Field::ULong(v) => *v as f64,
                        Field::Float(v) => *v as f64,
                        Field::Double(v) => *v,
                        Field::Null => f64::NAN, // Explicitly handle Null as NaN
                        _ => {
                            return Err(DataLoadingError::OsError(format!(
                                "Unsupported data type in column '{}' (index {}): {:?}",
                                name, i, field
                            )));
                        }
                    };
                    row_values.push(val);
                }

                if width == 0 {
                    width = row_values.len();
                } else if width != row_values.len() {
                    return Err(DataLoadingError::OsError(
                        "Inconsistent number of columns in Parquet file".to_string(),
                    ));
                }
                data.extend(row_values);
            }

            if width == 0 {
                return CausalTensor::new(Vec::new(), vec![0, 0])
                    .map_err(|e| DataLoadingError::OsError(e.to_string()));
            }

            let height = data.len() / width;
            CausalTensor::new(data, vec![height, width])
                .map_err(|e| DataLoadingError::OsError(e.to_string()))
        } else {
            Err(DataLoadingError::OsError(
                "Invalid config type for ParquetDataLoader".to_string(),
            ))
        }
    }
}
