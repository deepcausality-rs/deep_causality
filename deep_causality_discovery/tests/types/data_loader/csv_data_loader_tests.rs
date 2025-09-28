/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{
    CsvConfig, CsvDataLoader, DataLoader, DataLoaderConfig, DataLoadingError,
};
use deep_causality_tensor::CausalTensor;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_temp_csv(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file
}

#[test]
fn test_csv_data_loader_load_success_no_headers() {
    let csv_content = "1.0,2.0\n3.0,4.0\n5.0,6.0";
    let temp_file = create_temp_csv(csv_content);
    let path = temp_file.path().to_str().unwrap();

    let loader = CsvDataLoader;
    let csv_config = CsvConfig::new(false, b',', 0, None);
    let config = DataLoaderConfig::Csv(csv_config);

    let result = loader.load(path, &config).unwrap();
    let expected = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![3, 2]).unwrap();

    assert_eq!(result.as_slice(), expected.as_slice());
    assert_eq!(result.shape(), expected.shape());
}

#[test]
fn test_csv_data_loader_load_success_with_headers() {
    let csv_content = "col1,col2\n1.0,2.0\n3.0,4.0";
    let temp_file = create_temp_csv(csv_content);
    let path = temp_file.path().to_str().unwrap();

    let loader = CsvDataLoader;
    let csv_config = CsvConfig::new(true, b',', 0, None);
    let config = DataLoaderConfig::Csv(csv_config);

    let result = loader.load(path, &config).unwrap();
    let expected = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();

    assert_eq!(result.as_slice(), expected.as_slice());
    assert_eq!(result.shape(), expected.shape());
}

#[test]
fn test_csv_data_loader_load_success_with_delimiter() {
    let csv_content = "1.0;2.0\n3.0;4.0";
    let temp_file = create_temp_csv(csv_content);
    let path = temp_file.path().to_str().unwrap();

    let loader = CsvDataLoader;
    let csv_config = CsvConfig::new(false, b';', 0, None);
    let config = DataLoaderConfig::Csv(csv_config);

    let result = loader.load(path, &config).unwrap();
    let expected = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();

    assert_eq!(result.as_slice(), expected.as_slice());
    assert_eq!(result.shape(), expected.shape());
}

#[test]
fn test_csv_data_loader_load_success_with_skip_rows() {
    let csv_content = "header1,header2\nskip_this_row\n1.0,2.0\n3.0,4.0";
    let temp_file = create_temp_csv(csv_content);
    let path = temp_file.path().to_str().unwrap();

    let loader = CsvDataLoader;
    let csv_config = CsvConfig::new(true, b',', 1, None);
    let config = DataLoaderConfig::Csv(csv_config);

    let result = loader.load(path, &config).unwrap();
    let expected = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();

    assert_eq!(result.as_slice(), expected.as_slice());
    assert_eq!(result.shape(), expected.shape());
}

#[test]
fn test_csv_data_loader_load_error_file_not_found() {
    let loader = CsvDataLoader;
    let csv_config = CsvConfig::new(false, b',', 0, None);
    let config = DataLoaderConfig::Csv(csv_config);

    let result = loader.load("non_existent_file.csv", &config);
    assert!(result.is_err());
    if let Err(DataLoadingError::FileNotFound(path)) = result {
        assert_eq!(path, "non_existent_file.csv");
    } else {
        panic!("Expected FileNotFoundError, got {:?}", result);
    }
}

#[test]
fn test_csv_data_loader_load_error_invalid_data_format() {
    let csv_content = "1.0,abc\n3.0,4.0";
    let temp_file = create_temp_csv(csv_content);
    let path = temp_file.path().to_str().unwrap();

    let loader = CsvDataLoader;
    let csv_config = CsvConfig::new(false, b',', 0, None);
    let config = DataLoaderConfig::Csv(csv_config);

    let result = loader.load(path, &config);
    assert!(result.is_err());
    if let Err(DataLoadingError::OsError(e)) = result {
        assert!(e.contains("invalid float literal"));
    } else {
        panic!("Expected OsError with parse error, got {:?}", result);
    }
}

#[test]
fn test_csv_data_loader_load_error_invalid_config_type() {
    let csv_content = "1.0,2.0\n3.0,4.0";
    let temp_file = create_temp_csv(csv_content);
    let path = temp_file.path().to_str().unwrap();

    let loader = CsvDataLoader;
    // Provide a ParquetConfig instead of CsvConfig
    let parquet_config = deep_causality_discovery::ParquetConfig::new(None, 1024);
    let config = DataLoaderConfig::Parquet(parquet_config);

    let result = loader.load(path, &config);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        DataLoadingError::OsError("Invalid config type for CsvDataLoader".to_string())
    );
}

#[test]
fn test_csv_data_loader_load_empty_file() {
    let csv_content = "";
    let temp_file = create_temp_csv(csv_content);
    let path = temp_file.path().to_str().unwrap();

    let loader = CsvDataLoader;
    let csv_config = CsvConfig::new(false, b',', 0, None);
    let config = DataLoaderConfig::Csv(csv_config);

    let result = loader.load(path, &config).unwrap();
    let expected = CausalTensor::new(vec![], vec![0, 0]).unwrap();

    assert_eq!(result.as_slice(), expected.as_slice());
    assert_eq!(result.shape(), expected.shape());
}

#[test]
fn test_csv_data_loader_load_single_row_single_column() {
    let csv_content = "10.0";
    let temp_file = create_temp_csv(csv_content);
    let path = temp_file.path().to_str().unwrap();

    let loader = CsvDataLoader;
    let csv_config = CsvConfig::new(false, b',', 0, None);
    let config = DataLoaderConfig::Csv(csv_config);

    let result = loader.load(path, &config).unwrap();
    let expected = CausalTensor::new(vec![10.0], vec![1, 1]).unwrap();

    assert_eq!(result.as_slice(), expected.as_slice());
    assert_eq!(result.shape(), expected.shape());
}
