/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{CdlBuilder, CdlError};
use std::io::Write;
use tempfile::NamedTempFile;

fn create_temp_file(content: &str, extension: &str) -> NamedTempFile {
    let mut builder = tempfile::Builder::new();
    builder.suffix(extension);
    let mut file = builder.tempfile().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file
}

#[test]
fn test_load_data_csv_success() {
    let content = "a,b\n1,2";
    let file = create_temp_file(content, ".csv");
    let path = file.path().to_str().unwrap();

    let res = CdlBuilder::build().bind(|cdl| cdl.load_data(path, 1, vec![]));

    assert!(res.inner.is_ok());
    assert_eq!(res.inner.unwrap().state.records_count, 1);
}

#[test]
fn test_load_data_parquet_failure() {
    // Test dispatch to parquet loader with non-existent file
    let res = CdlBuilder::build().bind(|cdl| cdl.load_data("missing.parquet", 0, vec![]));

    assert!(res.inner.is_err());
}

#[test]
fn test_load_data_unsupported_extension() {
    let res = CdlBuilder::build().bind(|cdl| cdl.load_data("data.txt", 0, vec![]));

    assert!(res.inner.is_err());
    match res.inner {
        Err(CdlError::ReadDataError(_)) => {}
        _ => panic!("Expected ReadDataError"),
    }
}

#[test]
fn test_load_data_no_extension() {
    let res = CdlBuilder::build().bind(|cdl| cdl.load_data("data", 0, vec![]));

    assert!(res.inner.is_err());
    match res.inner {
        Err(CdlError::ReadDataError(_)) => {}
        _ => panic!("Expected ReadDataError"),
    }
}

#[test]
fn test_load_data_with_config_csv_success() {
    use deep_causality_discovery::{CsvConfig, DataLoaderConfig};

    let content = "a,b\n1,2";
    let file = create_temp_file(content, ".csv");
    let path = file.path().to_str().unwrap().to_string();

    let csv_config = CsvConfig::new(true, b',', 0, None, Some(path), Some(1), vec![]);
    let config = DataLoaderConfig::Csv(csv_config);

    let res = CdlBuilder::build().bind(|cdl| cdl.load_data_with_config(config.clone()));

    assert!(res.inner.is_ok());
    assert_eq!(res.inner.unwrap().state.records_count, 1);
}

#[test]
fn test_load_data_with_config_missing_path() {
    use deep_causality_discovery::{CsvConfig, DataLoaderConfig};

    let csv_config = CsvConfig::default(); // default has no path
    let config = DataLoaderConfig::Csv(csv_config);

    let res = CdlBuilder::build().bind(|cdl| cdl.load_data_with_config(config.clone()));

    assert!(res.inner.is_err());
}

#[test]
fn test_load_data_with_config_parquet_missing_path() {
    use deep_causality_discovery::{DataLoaderConfig, ParquetConfig};

    let parquet_config = ParquetConfig::default(); // default has no path
    let config = DataLoaderConfig::Parquet(parquet_config);

    let res = CdlBuilder::build().bind(|cdl| cdl.load_data_with_config(config.clone()));

    assert!(res.inner.is_err());
}

#[test]
fn test_load_data_with_config_csv_error() {
    use deep_causality_discovery::{CsvConfig, DataLoaderConfig};

    // Path pointing to non-existent file
    let path = "non_existent_file.csv".to_string();
    let csv_config = CsvConfig::new(true, b',', 0, None, Some(path), Some(1), vec![]);
    let config = DataLoaderConfig::Csv(csv_config);

    let res = CdlBuilder::build().bind(|cdl| cdl.load_data_with_config(config.clone()));

    assert!(res.inner.is_err());
}

#[test]
fn test_load_tensor_success() {
    use deep_causality_tensor::CausalTensor;

    // Create a 2x2 tensor
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();

    let res = CdlBuilder::build().bind(|cdl| cdl.load_tensor(tensor.clone()));

    assert!(res.inner.is_ok());

    let cdl_with_data = res.inner.unwrap();
    assert_eq!(cdl_with_data.state.records_count, 2);
    assert_eq!(cdl_with_data.state.tensor.shape(), &[2, 2]);
}
