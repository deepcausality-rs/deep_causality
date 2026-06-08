/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{CdlConfigBuilder, MaxOrder, SurdAnalyzeConfig};
use std::io::Write;
use tempfile::NamedTempFile;

fn temp() -> NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    f.write_all(b"a,b\n1.0,2.0\n").unwrap();
    f
}

#[test]
fn test_surd_loader_config_getters() {
    let file = temp();
    let path = file.path().to_str().unwrap().to_string();
    let config = CdlConfigBuilder::build_surd_config::<f64>()
        .with_path(&path)
        .with_target_index(3)
        .with_num_features(2)
        .with_max_order(MaxOrder::Max)
        .with_analyze(SurdAnalyzeConfig::new(0.01, 0.02, 0.03))
        .with_exclude_indices(vec![0])
        .build()
        .expect("file exists");

    assert_eq!(config.path(), path);
    assert_eq!(config.target_index(), 3);
    assert_eq!(config.num_features(), 2);
    assert!(matches!(config.max_order(), MaxOrder::Max));
    assert_eq!(config.exclude_indices(), &[0]);
    assert_eq!(config.analyze().unique_threshold(), 0.02);
    assert!(format!("{}", config).contains("SurdLoaderConfig"));
}

#[test]
fn test_surd_loader_config_default_exclude_is_empty() {
    let file = temp();
    let config = CdlConfigBuilder::build_surd_config::<f64>()
        .with_path(file.path().to_str().unwrap())
        .with_target_index(1)
        .with_num_features(1)
        .with_max_order(MaxOrder::Max)
        .with_analyze(SurdAnalyzeConfig::default())
        .build()
        .expect("file exists");
    assert!(config.exclude_indices().is_empty());
}
