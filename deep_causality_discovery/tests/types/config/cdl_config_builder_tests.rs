/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The staged builders enforce required fields at compile time; `build()` adds a
//! runtime file-exists check.

use deep_causality_discovery::{
    BrcdConfig, CdlConfigBuilder, CdlError, MaxOrder, SurdAnalyzeConfig,
};
use std::io::Write;
use tempfile::NamedTempFile;

fn temp() -> NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    f.write_all(b"x,y\n1.0,2.0\n").unwrap();
    f
}

#[test]
fn test_surd_build_fails_when_file_missing() {
    let res = CdlConfigBuilder::build_surd_config::<f64>()
        .with_path("/no/such/surd_file.csv")
        .with_target_index(0)
        .with_num_features(1)
        .with_max_order(MaxOrder::Max)
        .with_analyze(SurdAnalyzeConfig::default())
        .build();
    assert!(matches!(res, Err(CdlError::ReadDataError(_))));
}

#[test]
fn test_brcd_build_fails_when_normal_missing() {
    let a = temp();
    let res = CdlConfigBuilder::build_brcd_config()
        .with_normal_path("/no/such/normal.csv")
        .with_anomalous_path(a.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(0))
        .build();
    assert!(matches!(res, Err(CdlError::ReadDataError(_))));
}

#[test]
fn test_brcd_build_fails_when_cpdag_missing() {
    let n = temp();
    let a = temp();
    let res = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(n.path().to_str().unwrap())
        .with_anomalous_path(a.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(0))
        .with_cpdag_path("/no/such/cpdag.csv")
        .build();
    assert!(matches!(res, Err(CdlError::ReadDataError(_))));
}

#[test]
fn test_brcd_build_succeeds_when_files_exist() {
    let n = temp();
    let a = temp();
    let res = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(n.path().to_str().unwrap())
        .with_anomalous_path(a.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(0))
        .build();
    assert!(res.is_ok());
}
