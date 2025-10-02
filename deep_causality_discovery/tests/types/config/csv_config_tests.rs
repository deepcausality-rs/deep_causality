/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::CsvConfig;

#[test]
fn test_new_and_getters() {
    let columns = vec!["a".to_string(), "b".to_string()];
    let config = CsvConfig::new(true, b';', 1, Some(columns.clone()));

    assert!(config.has_headers());
    assert_eq!(config.delimiter(), b';');
    assert_eq!(config.skip_rows(), 1);
    assert_eq!(config.columns(), &Some(columns));
}

#[test]
fn test_new_no_columns() {
    let config = CsvConfig::new(false, b',', 0, None);
    assert!(!config.has_headers());
    assert_eq!(config.delimiter(), b',');
    assert_eq!(config.skip_rows(), 0);
    assert_eq!(config.columns(), &None);
}

#[test]
fn test_display_with_columns() {
    let columns = vec!["a".to_string(), "b".to_string()];
    let config = CsvConfig::new(true, b',', 0, Some(columns));
    let display = format!("{}", config);
    assert_eq!(
        display,
        "CsvConfig(headers: true, delimiter: ,, skip: 0, columns: Some([\"a\", \"b\"]))"
    );
}

#[test]
fn test_display_no_columns() {
    let config = CsvConfig::new(false, b'|', 5, None);
    let display = format!("{}", config);
    assert_eq!(
        display,
        "CsvConfig(headers: false, delimiter: |, skip: 5, columns: None)"
    );
}

#[test]
fn test_clone() {
    let columns = vec!["a".to_string(), "b".to_string()];
    let config1 = CsvConfig::new(true, b',', 0, Some(columns));
    let config2 = config1.clone();
    assert_eq!(config1.has_headers(), config2.has_headers());
    assert_eq!(config1.delimiter(), config2.delimiter());
    assert_eq!(config1.skip_rows(), config2.skip_rows());
    assert_eq!(config1.columns(), config2.columns());
}

#[test]
fn test_debug() {
    let config = CsvConfig::new(true, b',', 0, None);
    let debug = format!("{:?}", config);
    assert!(debug.starts_with("CsvConfig"));
    assert!(debug.contains("has_headers: true"));
    assert!(debug.contains("delimiter: 44")); // ASCII for ','
    assert!(debug.contains("skip_rows: 0"));
    assert!(debug.contains("columns: None"));
}

#[test]
fn test_default() {
    let config = CsvConfig::default();
    assert!(config.has_headers());
    assert_eq!(config.delimiter(), b',');
    assert_eq!(config.skip_rows(), 0);
    assert_eq!(config.columns(), &None);
}
