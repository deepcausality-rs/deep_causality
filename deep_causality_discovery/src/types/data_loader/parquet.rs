/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::DataError;
use crate::traits::process_data_loader::ProcessDataLoader;
use crate::types::config::DataLoaderConfig;
use deep_causality_tensor::CausalTensor;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;
use std::fs::File;
use std::path::Path;

/// A concrete implementation of `ProcessDataLoader` for reading data from Parquet files.
pub struct ParquetDataLoader;

impl ProcessDataLoader for ParquetDataLoader {
    fn load(&self, path: &str, config: &DataLoaderConfig) -> Result<CausalTensor<f64>, DataError> {
        if let DataLoaderConfig::Parquet(_parquet_config) = config {
            let file =
                File::open(Path::new(path)).map_err(|e| DataError::OsError(e.to_string()))?;
            let reader = SerializedFileReader::new(file)?;

            let mut data = Vec::new();
            let mut width = 0;

            let iter = reader.get_row_iter(None)?;

            for record_result in iter {
                let record = record_result?;
                let fields: Vec<f64> = record
                    .get_column_iter()
                    .map(|(_name, field)| {
                        match field {
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
                            _ => f64::NAN, // Mark unsupported types
                        }
                    })
                    .collect();

                if fields.iter().any(|v| v.is_nan()) {
                    return Err(DataError::OsError(
                        "Unsupported data type in Parquet file".to_string(),
                    ));
                }

                if width == 0 {
                    width = fields.len();
                } else if width != fields.len() {
                    return Err(DataError::OsError(
                        "Inconsistent number of columns in Parquet file".to_string(),
                    ));
                }
                data.extend(fields);
            }

            if width == 0 {
                return CausalTensor::new(Vec::new(), vec![0, 0])
                    .map_err(|e| DataError::OsError(e.to_string()));
            }

            let height = data.len() / width;
            CausalTensor::new(data, vec![height, width])
                .map_err(|e| DataError::OsError(e.to_string()))
        } else {
            Err(DataError::OsError(
                "Invalid config type for ParquetDataLoader".to_string(),
            ))
        }
    }
}
