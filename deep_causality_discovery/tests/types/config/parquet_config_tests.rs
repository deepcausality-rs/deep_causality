/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::ParquetConfig;

#[test]
fn test_new_and_getters() {
    let columns = vec!["x".to_string(), "y".to_string()];
    let config = ParquetConfig::new(Some(columns.clone()), 2048);
    assert_eq!(config.columns(), &Some(columns));
    assert_eq!(config.batch_size(), 2048);
}

#[test]
fn test_default() {
    let config = ParquetConfig::default();
    assert_eq!(config.columns(), &None);
    assert_eq!(config.batch_size(), 1024);
}

#[test]
fn test_display_with_columns() {
    let columns = vec!["x".to_string(), "y".to_string()];
    let config = ParquetConfig::new(Some(columns), 2048);
    let display = format!("{}", config);
    assert_eq!(
        display,
        "ParquetConfig(columns: Some([\"x\", \"y\"]), batch_size: 2048)"
    );
}

#[test]
fn test_display_no_columns() {
    let config = ParquetConfig::new(None, 512);
    let display = format!("{}", config);
    assert_eq!(display, "ParquetConfig(columns: None, batch_size: 512)");
}

#[test]
fn test_clone() {
    let columns = vec!["x".to_string(), "y".to_string()];
    let config1 = ParquetConfig::new(Some(columns), 2048);
    let config2 = config1.clone();
    assert_eq!(config1.columns(), config2.columns());
    assert_eq!(config1.batch_size(), config2.batch_size());
}

#[test]
fn test_debug() {
    let config = ParquetConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.starts_with("ParquetConfig"));
    assert!(debug.contains("columns: None"));
    assert!(debug.contains("batch_size: 1024"));
}
