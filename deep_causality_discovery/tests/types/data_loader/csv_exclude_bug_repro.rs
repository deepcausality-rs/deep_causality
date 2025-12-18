/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{CsvConfig, CsvDataLoader, DataLoader, DataLoaderConfig};
use deep_causality_tensor::CausalTensor;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_temp_csv(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file
}

#[test]
fn test_csv_exclude_indices_bug() {
    // CSV with 4 columns
    let csv_content = "1.0,2.0,3.0,4.0\n5.0,6.0,7.0,8.0";
    let temp_file = create_temp_csv(csv_content);
    let path = temp_file.path().to_str().unwrap();

    let loader = CsvDataLoader;
    // Configure to exclude columns at indices 1 and 2
    let csv_config = CsvConfig::new(
        false,      // no headers
        b',',       // comma delimiter
        0,          // skip no rows
        None,       // all columns
        None,       // no file path
        None,       // no target index
        vec![1, 2], // exclude indices 1 and 2
    );
    let config = DataLoaderConfig::Csv(csv_config);

    let result = loader.load(path, &config).unwrap();

    // Expected: Only columns 0 and 3 should be loaded
    // Row 1: 1.0, 4.0
    // Row 2: 5.0, 8.0
    let expected = CausalTensor::new(vec![1.0, 4.0, 5.0, 8.0], vec![2, 2]).unwrap();

    println!("Result shape: {:?}", result.shape());
    println!("Result data: {:?}", result.as_slice());
    println!("Expected shape: {:?}", expected.shape());
    println!("Expected data: {:?}", expected.as_slice());

    // This assertion will FAIL because CSV loader ignores exclude_indices
    assert_eq!(result.shape(), expected.shape(), "Shape mismatch");
    assert_eq!(result.as_slice(), expected.as_slice(), "Data mismatch");
}
