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
