/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{CsvConfig, DataLoaderConfig, ParquetConfig};

#[test]
fn test_display_csv() {
    let csv_config = CsvConfig::new(true, b',', 0, None, None, None, vec![]);
    let config = DataLoaderConfig::Csv(csv_config);
    let display = format!("{}", config);
    assert_eq!(
        display,
        "DataLoaderConfig::Csv(CsvConfig(headers: true, delimiter: ,, skip: 0, columns: None))"
    );
}

#[test]
fn test_display_parquet() {
    let parquet_config = ParquetConfig::default();
    let config = DataLoaderConfig::Parquet(parquet_config);
    let display = format!("{}", config);
    assert_eq!(
        display,
        "DataLoaderConfig::Parquet(ParquetConfig(columns: None, batch_size: 1024))"
    );
}

#[test]
fn test_clone() {
    let csv_config = CsvConfig::new(true, b',', 0, None, None, None, vec![]);
    let config1 = DataLoaderConfig::Csv(csv_config);
    let config2 = config1.clone();
    match (config1, config2) {
        (DataLoaderConfig::Csv(c1), DataLoaderConfig::Csv(c2)) => {
            assert_eq!(c1.has_headers(), c2.has_headers());
        }
        _ => panic!("Clone failed"),
    }
}

#[test]
fn test_debug() {
    let parquet_config = ParquetConfig::default();
    let config = DataLoaderConfig::Parquet(parquet_config);
    let debug = format!("{:?}", config);
    assert!(debug.starts_with("Parquet(ParquetConfig"));
    assert!(debug.contains("columns: None"));
    assert!(debug.contains("batch_size: 1024"));
}
