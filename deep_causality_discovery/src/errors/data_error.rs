/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

#[derive(Debug)]
pub enum DataError {
    FileNotFound(String),
    PermissionDenied(String),
    OsError(String),
    CsvError(csv::Error),
    ParquetError(parquet::errors::ParquetError),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataError::FileNotFound(s) => write!(f, "File not found at path: {}", s),
            DataError::PermissionDenied(s) => write!(f, "Permission denied: {}", s),
            DataError::OsError(s) => write!(f, "OS error: {}", s),
            DataError::CsvError(e) => write!(f, "CSV parsing error: {}", e),
            DataError::ParquetError(e) => write!(f, "Parquet parsing error: {}", e),
        }
    }
}

impl std::error::Error for DataError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DataError::CsvError(e) => Some(e),
            DataError::ParquetError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<csv::Error> for DataError {
    fn from(err: csv::Error) -> DataError {
        DataError::CsvError(err)
    }
}
impl From<parquet::errors::ParquetError> for DataError {
    fn from(err: parquet::errors::ParquetError) -> DataError {
        DataError::ParquetError(err)
    }
}
