/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum DataLoadingError {
    FileNotFound(String),
    PermissionDenied(String),
    OsError(String),
    CsvError(String),
    ParquetError(String),
}

impl fmt::Display for DataLoadingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataLoadingError::FileNotFound(s) => write!(f, "File not found at path: {}", s),
            DataLoadingError::PermissionDenied(s) => write!(f, "Permission denied: {}", s),
            DataLoadingError::OsError(s) => write!(f, "OS error: {}", s),
            DataLoadingError::CsvError(e) => write!(f, "CSV parsing error: {}", e),
            DataLoadingError::ParquetError(e) => write!(f, "Parquet parsing error: {}", e),
        }
    }
}

impl std::error::Error for DataLoadingError {}

impl From<csv::Error> for DataLoadingError {
    fn from(err: csv::Error) -> DataLoadingError {
        DataLoadingError::CsvError(err.to_string())
    }
}
impl From<parquet::errors::ParquetError> for DataLoadingError {
    fn from(err: parquet::errors::ParquetError) -> DataLoadingError {
        DataLoadingError::ParquetError(err.to_string())
    }
}
