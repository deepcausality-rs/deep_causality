/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::CsvConfig;

#[test]
fn test_csv_config_new() {
    let config = CsvConfig::new(
        true,
        b';',
        1,
        Some(vec!["A".to_string()]),
        Some("/tmp/data.csv".into()),
        Some(3),
        vec![],
    );
    assert!(config.has_headers());
    assert_eq!(config.target_index(), Some(3));
    assert!(config.exclude_indices().is_empty());
    assert_eq!(config.delimiter(), b';');
    assert_eq!(config.skip_rows(), 1);
    assert_eq!(config.columns().as_ref().unwrap()[0], "A");
    assert_eq!(config.file_path().unwrap(), "/tmp/data.csv");
}

#[test]
fn test_csv_config_default() {
    let config = CsvConfig::default();
    assert!(config.has_headers());
    assert_eq!(config.delimiter(), b',');
    assert_eq!(config.skip_rows(), 0);
    assert!(config.columns().is_none());
    assert!(config.file_path().is_none());
    assert!(config.target_index().is_none());
    assert!(config.exclude_indices().is_empty());
}

#[test]
fn test_csv_config_display() {
    let config = CsvConfig::new(true, b',', 0, None, None, None, vec![]);
    let s = format!("{}", config);
    assert!(s.contains("CsvConfig"));
    assert!(s.contains("headers: true"));
}
