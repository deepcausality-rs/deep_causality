/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{
    DataError, DataLoaderConfig, ParquetConfig, ParquetDataLoader, ProcessDataLoader,
};
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_parquet_data_loader_load_error_file_not_found() {
    let loader = ParquetDataLoader;
    let parquet_config = ParquetConfig::new(None, 1024);
    let config = DataLoaderConfig::Parquet(parquet_config);

    let result = loader.load("non_existent_file.parquet", &config);
    assert!(result.is_err());
    if let Err(DataError::OsError(e)) = result {
        assert!(e.contains("No such file or directory"));
    } else {
        panic!("Expected OsError, got {:?}", result);
    }
}

#[test]
fn test_parquet_data_loader_load_error_invalid_config_type() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();
    // Create a dummy file to ensure File::open succeeds, but it's not a valid Parquet file.
    // The error should come from the DataLoaderConfig mismatch.
    fs::write(path, "dummy content").unwrap();

    let loader = ParquetDataLoader;
    // Provide a CsvConfig instead of ParquetConfig
    let csv_config = deep_causality_discovery::CsvConfig::new(false, b',', 0, None);
    let config = DataLoaderConfig::Csv(csv_config);

    let result = loader.load(path, &config);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        DataError::OsError("Invalid config type for ParquetDataLoader".to_string())
    );
}
