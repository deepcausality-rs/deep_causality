/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::ParquetConfig;

#[test]
fn test_parquet_config_new() {
    let config = ParquetConfig::new(
        Some(vec!["A".to_string()]),
        1024,
        Some("/tmp/data.parquet".into()),
        Some(3),
        vec![1, 2],
    );
    assert_eq!(config.columns().as_ref().unwrap()[0], "A");
    assert_eq!(config.target_index(), Some(3));
    assert_eq!(config.exclude_indices().len(), 2);
    assert_eq!(config.batch_size(), 1024);
    assert_eq!(config.file_path().unwrap(), "/tmp/data.parquet");
}

#[test]
fn test_parquet_config_default() {
    let config = ParquetConfig::default();
    assert!(config.columns().is_none());
    assert_eq!(config.batch_size(), 1024);
    assert!(config.file_path().is_none());
    assert!(config.target_index().is_none());
    assert!(config.exclude_indices().is_empty());
}

#[test]
fn test_parquet_config_display() {
    let config = ParquetConfig::new(None, 2048, None, None, vec![]);
    let s = format!("{}", config);
    assert!(s.contains("ParquetConfig"));
    assert!(s.contains("batch_size: 2048"));
}
