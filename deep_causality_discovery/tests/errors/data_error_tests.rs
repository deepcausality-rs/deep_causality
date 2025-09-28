/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::DataLoadingError;
use std::error::Error;
use std::io;

#[test]
fn test_display() {
    let err = DataLoadingError::FileNotFound("non_existent.csv".to_string());
    assert_eq!(err.to_string(), "File not found at path: non_existent.csv");

    let err = DataLoadingError::PermissionDenied("restricted_file.txt".to_string());
    assert_eq!(err.to_string(), "Permission denied: restricted_file.txt");

    let err = DataLoadingError::OsError("some os error".to_string());
    assert_eq!(err.to_string(), "OS error: some os error");

    let csv_err = csv::Error::from(io::Error::other("test io error"));
    let err = DataLoadingError::CsvError(csv_err.to_string());
    assert!(err.to_string().contains("CSV parsing error:"));

    // ParquetError is harder to construct directly without a real file operation.
    // We'll create a dummy one for testing purposes.
    let parquet_err = parquet::errors::ParquetError::General("dummy parquet error".to_string());
    let err = DataLoadingError::ParquetError(parquet_err.to_string());
    assert!(err.to_string().contains("Parquet parsing error:"));
}

#[test]
fn test_source() {
    let err = DataLoadingError::FileNotFound("non_existent.csv".to_string());
    assert!(err.source().is_none());

    let err = DataLoadingError::PermissionDenied("restricted_file.txt".to_string());
    assert!(err.source().is_none());

    let err = DataLoadingError::OsError("some OS error".to_string());
    assert!(err.source().is_none());

    let csv_err = csv::Error::from(io::Error::other("test io error"));
    let err = DataLoadingError::CsvError(csv_err.to_string());
    assert!(err.source().is_none());

    let parquet_err = parquet::errors::ParquetError::General("dummy parquet error".to_string());
    let err = DataLoadingError::ParquetError(parquet_err.to_string());
    assert!(err.source().is_none());
}

#[test]
fn test_from_csv_error() {
    let csv_err = csv::Error::from(io::Error::other("test io error"));
    let err = DataLoadingError::from(csv_err);
    if let DataLoadingError::CsvError(_) = err {
        // Test passed
    } else {
        panic!("Incorrect error variant for csv::Error");
    }
}

#[test]
fn test_from_parquet_error() {
    let parquet_err = parquet::errors::ParquetError::General("dummy parquet error".to_string());
    let err = DataLoadingError::from(parquet_err);
    if let DataLoadingError::ParquetError(_) = err {
        // Test passed
    } else {
        panic!("Incorrect error variant for parquet::errors::ParquetError");
    }
}
